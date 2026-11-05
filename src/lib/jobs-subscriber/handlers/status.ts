/**
 * status handler — jobs-status
 *
 * Success: validate → fn_pipeline_apply_status (preserves eventId) → ACK
 * Failure: log; no status DLQ — rely on Kafka redelivery / no ACK
 */

import { isoNowIST } from "@/lib/isoNowIST";
import { applyStatus } from "@/lib/jobs-subscriber/postgres/apply-status";
import { pipelineStatusEventSchema } from "@/schemas/pipeline-status.schema";
import type { InboundEventMeta, StatusHandlerDeps } from "@/types/global.type";

export async function handleStatusEvent(
  payload: Record<string, unknown>,
  _meta: InboundEventMeta,
  deps: StatusHandlerDeps,
): Promise<void> {
  const transport = deps.getTransport();
  if (!transport) {
    console.error(`${isoNowIST()}\t[JobsSub:Status]\ttransport not ready`);
    return;
  }

  let correlationId: string | undefined;

  try {
    const event = pipelineStatusEventSchema.parse(payload);
    correlationId = event.correlationId;

    const result = await applyStatus({
      eventId: event.eventId,
      correlationId: event.correlationId,
      stage: event.stage,
      status: event.status,
      timestamp: event.timestamp,
      details: event.details,
    });

    transport.sendProcessed(correlationId);
    console.log(
      `${isoNowIST()}\t[JobsSub:Status]\tOK corr=${correlationId} stage=${event.stage} seq=${result.assignedSequence} replayed=${result.replayed}`,
    );
  } catch (err: unknown) {
    const reason = err instanceof Error ? err.message : String(err);
    console.error(
      `${isoNowIST()}\t[JobsSub:Status]\tFAIL corr=${correlationId ?? "?"}\terr=${reason}`,
    );
    // No ACK → Kafka / gateway redelivery. No status DLQ.
  }
}
