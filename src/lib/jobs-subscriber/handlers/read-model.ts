/**
 * read-model handler — jobs-ingest-read-model
 *
 * Success path:
 *   1. validate event
 *   2. status READ_MODEL_STARTED
 *   3. PostgreSQL SP (event.job.readModel) — does NOT query Azure
 *   4. status READ_MODEL_SUCCEEDED
 *   5. status COMPLETED
 *   6. ACK
 *
 * Failure: FAILED status → retry/DLQ → ACK after publish succeeds.
 */

import { isoNowIST } from "@/lib/isoNowIST";
import { publishRetryOrDlq } from "@/lib/jobs-subscriber/failure";
import { executeReadModel } from "@/lib/jobs-subscriber/postgres/execute-read-model";
import { getRetryCount } from "@/lib/jobs-subscriber/retry-policy";
import {
  publishStatus,
  tryPublishFailedStatus,
} from "@/lib/jobs-subscriber/status-publish";
import { JOBS_TOPICS } from "@/lib/jobs-subscriber/topics";
import { jobsEventSchema } from "@/schemas/jobs-event.schema";
import type { InboundEventMeta, ReadModelHandlerDeps } from "@/types/global.type";
import { z } from "zod";

const correlationIdSchema = z
  .object({
    correlationId: z.string().min(1),
  })
  .passthrough();

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

export async function handleReadModelEvent(
  payload: Record<string, unknown>,
  meta: InboundEventMeta,
  deps: ReadModelHandlerDeps,
): Promise<void> {
  const transport = deps.getTransport();
  if (!transport) {
    console.error(`${isoNowIST()}\t[JobsSub:ReadModel]\ttransport not ready`);
    return;
  }

  const retryCount = getRetryCount(meta.headers);
  const original = domainPayload(payload);

  const idParse = correlationIdSchema.safeParse(payload);
  let correlationId = idParse.success
    ? idParse.data.correlationId
    : undefined;

  try {
    const event = jobsEventSchema.parse(payload);
    correlationId = event.correlationId;

    await publishStatus(
      transport,
      correlationId,
      "READ_MODEL_STARTED",
      "STARTED",
      {
        message: "PostgreSQL read-model starting",
        procedure: event.job.readModel,
        recordCount: event.records.length,
        retryCount,
      },
    );

    const result = await executeReadModel({
      procedureName: event.job.readModel,
      correlationId,
      records: event.records,
    });

    await publishStatus(
      transport,
      correlationId,
      "READ_MODEL_SUCCEEDED",
      "SUCCEEDED",
      {
        message: result.replayed
          ? "Read-model already applied (replay)"
          : "Read-model applied successfully",
        rowsInserted: result.rowsInserted,
        replayed: result.replayed,
      },
    );

    await publishStatus(transport, correlationId, "COMPLETED", "SUCCEEDED", {
      message: "Pipeline completed",
      rowsInserted: result.rowsInserted,
    });

    transport.sendProcessed(correlationId);
    console.log(
      `${isoNowIST()}\t[JobsSub:ReadModel]\tOK corr=${correlationId} rows=${result.rowsInserted} replayed=${result.replayed}`,
    );
  } catch (err: unknown) {
    const reason = err instanceof Error ? err.message : String(err);
    if (!correlationId) {
      console.error(
        `${isoNowIST()}\t[JobsSub:ReadModel]\tFAIL no correlationId err=${reason}`,
      );
      return;
    }

    await tryPublishFailedStatus(
      transport,
      correlationId,
      "Read-model processing failed",
      reason,
      "JobsSub:ReadModel",
    );

    await publishRetryOrDlq({
      transport,
      stageTopic: JOBS_TOPICS.ingestReadModel,
      dlqTopic: JOBS_TOPICS.ingestReadModelDlq,
      payload: original,
      correlationId,
      currentRetryCount: retryCount,
      failureReason: reason,
      originalPartition: meta.partition,
      originalOffset: meta.offset,
      logPrefix: "JobsSub:ReadModel",
    });

    transport.sendProcessed(correlationId);
  }
}
