/**
 * Direct status publish to jobs-status.
 * eventId is generated once here and must survive Kafka retry/redelivery.
 * Failed status publication is logged and swallowed so it never hides
 * the original pipeline error or substitutes for an ACK.
 */

import type { WsPublishTransport } from "@/lib/gateway-ws/ws-publish-transport";
import { isoNowIST } from "@/lib/isoNowIST";
import { JOBS_TOPICS } from "@/lib/jobs-subscriber/topics";
import type {
  PipelineStage,
  PipelineStatusValue,
} from "@/schemas/pipeline-status.schema";
import { randomUUIDv7 } from "bun";

export async function publishStatus(
  transport: WsPublishTransport,
  correlationId: string,
  stage: PipelineStage,
  status: PipelineStatusValue,
  details?: Record<string, unknown>,
): Promise<void> {
  const eventId = randomUUIDv7();
  await transport.publish({
    topic: JOBS_TOPICS.status,
    key: correlationId,
    value: {
      eventId,
      correlationId,
      stage,
      status,
      timestamp: new Date().toISOString(),
      sequence: null,
      details,
    },
  });
}

/** Best-effort FAILED status; never throws. */
export async function tryPublishFailedStatus(
  transport: WsPublishTransport,
  correlationId: string,
  message: string,
  error: string,
  logPrefix: string,
): Promise<void> {
  try {
    await publishStatus(transport, correlationId, "FAILED", "FAILED", {
      message,
      error: error.slice(0, 500),
    });
  } catch (statusErr) {
    console.error(
      `${isoNowIST()}\t[${logPrefix}]\tFAILED status publish failed corr=${correlationId}`,
      statusErr,
    );
  }
}
