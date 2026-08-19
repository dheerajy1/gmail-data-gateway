/**
 * Stage-isolated retry / DLQ publication.
 * ACK remains the caller's responsibility after this succeeds.
 */

import { isoNowIST } from "@/lib/isoNowIST";
import {
  HEADER_CORRELATION_ID,
  HEADER_FAILURE_REASON,
  HEADER_ORIGINAL_OFFSET,
  HEADER_ORIGINAL_PARTITION,
  HEADER_ORIGINAL_TOPIC,
  MAX_RETRIES,
  RETRY_COUNT_HEADER,
  shouldDlq,
} from "@/lib/jobs-subscriber/retry-policy";
import type { FailureContext } from "@/types/global.type";

/**
 * - if retry count < MAX_RETRIES → republish to same stage topic with count+1
 * - else → publish to stage DLQ with metadata headers
 * Throws if publish fails (caller must not ACK).
 */
export async function publishRetryOrDlq(ctx: FailureContext): Promise<void> {
  const reason = ctx.failureReason.slice(0, 500);

  if (!shouldDlq(ctx.currentRetryCount)) {
    const nextRetryCount = ctx.currentRetryCount + 1;
    console.log(
      `${isoNowIST()}\t[${ctx.logPrefix}:Retry]\tcorr=${ctx.correlationId}\tcount=${nextRetryCount}/${MAX_RETRIES}\ttopic=${ctx.stageTopic}\treason=${reason}`,
    );
    await ctx.transport.publish({
      topic: ctx.stageTopic,
      key: ctx.correlationId,
      value: ctx.payload,
      headers: {
        [RETRY_COUNT_HEADER]: String(nextRetryCount),
        [HEADER_CORRELATION_ID]: ctx.correlationId,
        [HEADER_FAILURE_REASON]: reason,
      },
    });
    return;
  }

  console.log(
    `${isoNowIST()}\t[${ctx.logPrefix}:DLQ]\tcorr=${ctx.correlationId}\tfrom=${ctx.stageTopic}\tto=${ctx.dlqTopic}\treason=${reason}`,
  );

  const headers: Record<string, string> = {
    [RETRY_COUNT_HEADER]: String(ctx.currentRetryCount),
    [HEADER_ORIGINAL_TOPIC]: ctx.stageTopic,
    [HEADER_CORRELATION_ID]: ctx.correlationId,
    [HEADER_FAILURE_REASON]: reason,
  };
  if (ctx.originalPartition != null) {
    headers[HEADER_ORIGINAL_PARTITION] = String(ctx.originalPartition);
  }
  if (ctx.originalOffset != null) {
    headers[HEADER_ORIGINAL_OFFSET] = String(ctx.originalOffset);
  }

  await ctx.transport.publish({
    topic: ctx.dlqTopic,
    key: ctx.correlationId,
    value: ctx.payload,
    headers,
  });
}
