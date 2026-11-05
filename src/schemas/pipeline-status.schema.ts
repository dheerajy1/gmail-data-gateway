import { z } from "zod";

/** Pipeline stages published to jobs-status. */
export const PIPELINE_STAGES = [
  "RECEIVED",
  "AZURE_WRITE_STARTED",
  "AZURE_WRITE_SUCCEEDED",
  "READ_MODEL_STARTED",
  "READ_MODEL_SUCCEEDED",
  "COMPLETED",
  "FAILED",
] as const;

export type PipelineStage = (typeof PIPELINE_STAGES)[number];

export const PIPELINE_STATUS_VALUES = [
  "STARTED",
  "SUCCEEDED",
  "FAILED",
] as const;

export type PipelineStatusValue = (typeof PIPELINE_STATUS_VALUES)[number];

/**
 * Canonical pipeline status event.
 * eventId is generated once at publish time and preserved across Kafka redelivery.
 * sequence is assigned by the database (nullable on the wire).
 */
export const pipelineStatusEventSchema = z.object({
  eventId: z.uuid({ version: "v7" }),
  correlationId: z.uuid({ version: "v7" }),
  stage: z.enum(PIPELINE_STAGES),
  status: z.enum(PIPELINE_STATUS_VALUES),
  timestamp: z.string().min(10),
  sequence: z.number().nullable().optional(),
  details: z.record(z.string(), z.unknown()).optional(),
});

export type PipelineStatusEvent = z.infer<typeof pipelineStatusEventSchema>;
