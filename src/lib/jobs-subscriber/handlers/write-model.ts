/**
 * write-model handler — jobs-ingest-write-model
 *
 * Success path:
 *   1. validate event
 *   2. status RECEIVED
 *   3. status AZURE_WRITE_STARTED
 *   4. Azure SP (event.job.writeModel) with correlationId
 *   5. status AZURE_WRITE_SUCCEEDED (rowsInserted may be 0)
 *   6. publish same business event → jobs-ingest-read-model
 *   7. ACK
 *
 * Failure: FAILED status (best-effort) → retry/DLQ → ACK after publish succeeds.
 */

import { isoNowIST } from "@/lib/isoNowIST";
import { executeWriteModel } from "@/lib/jobs-subscriber/azure/execute-write-model";
import { publishRetryOrDlq } from "@/lib/jobs-subscriber/failure";
import { getRetryCount } from "@/lib/jobs-subscriber/retry-policy";
import {
  publishStatus,
  tryPublishFailedStatus,
} from "@/lib/jobs-subscriber/status-publish";
import { JOBS_TOPICS } from "@/lib/jobs-subscriber/topics";
import { jobsEventSchema } from "@/schemas/jobs-event.schema";
import type { InboundEventMeta, WriteModelHandlerDeps } from "@/types/global.type";

function domainPayload(
  payload: Record<string, unknown>,
): Record<string, unknown> {
  const {
    type: _type,
    topic: _topic,
    headers: _headers,
    partition: _partition,
    offset: _offset,
    ...rest
  } = payload;
  return rest;
}

export async function handleWriteModelEvent(
  payload: Record<string, unknown>,
  meta: InboundEventMeta,
  deps: WriteModelHandlerDeps,
): Promise<void> {
  const transport = deps.getTransport();
  if (!transport) {
    console.error(`${isoNowIST()}\t[JobsSub:WriteModel]\ttransport not ready`);
    return;
  }

  const retryCount = getRetryCount(meta.headers);
  const original = domainPayload(payload);
  let correlationId =
    typeof payload.correlationId === "string"
      ? payload.correlationId
      : undefined;

  try {
    const event = jobsEventSchema.parse(payload);
    correlationId = event.correlationId;

    await publishStatus(transport, correlationId, "RECEIVED", "SUCCEEDED", {
      message: "Valid write-model event accepted by subscriber",
      sourceClientId: event.sourceClientId,
      recordCount: event.records.length,
      retryCount,
    });

    await publishStatus(
      transport,
      correlationId,
      "AZURE_WRITE_STARTED",
      "STARTED",
      {
        message: "Azure write-model starting",
        procedure: event.job.writeModel,
        recordCount: event.records.length,
      },
    );

    const { data: spData } = await executeWriteModel({
      procedureName: event.job.writeModel,
      correlationId,
      records: event.records,
    });

    await publishStatus(
      transport,
      correlationId,
      "AZURE_WRITE_SUCCEEDED",
      "SUCCEEDED",
      {
        message: spData.replayed
          ? "Azure write already completed (replay)"
          : spData.rowsInserted === 0
            ? "Azure write succeeded with no new rows"
            : "Azure write-model succeeded",
        rowsInserted: spData.rowsInserted,
        replayed: spData.replayed,
      },
    );

    await transport.publish({
      topic: JOBS_TOPICS.ingestReadModel,
      key: correlationId,
      value: event,
    });

    transport.sendProcessed(correlationId);
    console.log(
      `${isoNowIST()}\t[JobsSub:WriteModel]\tOK corr=${correlationId} rows=${spData.rowsInserted} replayed=${Boolean(spData.replayed)}`,
    );
  } catch (err: unknown) {
    const reason = err instanceof Error ? err.message : String(err);
    if (!correlationId) {
      console.error(
        `${isoNowIST()}\t[JobsSub:WriteModel]\tFAIL no correlationId err=${reason}`,
      );
      return;
    }

    await tryPublishFailedStatus(
      transport,
      correlationId,
      "Write-model processing failed",
      reason,
      "JobsSub:WriteModel",
    );

    await publishRetryOrDlq({
      transport,
      stageTopic: JOBS_TOPICS.ingestWriteModel,
      dlqTopic: JOBS_TOPICS.ingestWriteModelDlq,
      payload: original,
      correlationId,
      currentRetryCount: retryCount,
      failureReason: reason,
      originalPartition: meta.partition,
      originalOffset: meta.offset,
      logPrefix: "JobsSub:WriteModel",
    });

    transport.sendProcessed(correlationId);
  }
}
