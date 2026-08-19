import { z } from "zod";

/** Single Jobs applied-mail record (Azure SP + Kafka read-model contract). */
export const jobsRecordSchema = z.object({
  id: z.string().min(1),
  date: z.string().min(1),
  from: z.string().min(1),
  subject: z.string().min(1),
  snippet: z.string().min(1),
});

export type JobsRecord = z.infer<typeof jobsRecordSchema>;

/**
 * Jobs domain event published as the Kafka message value.
 * Owned by jobs-api-gateway.
 * records use the same object-array contract as 02spinsertRawAppliedJobs.
 */
export const jobsEventSchema = z.object({
  correlationId: z.uuid({ version: "v7" }),
  job: z.object({
    writeModel: z.string().min(1),
    readModel: z.string().min(1),
  }),
  sourceClientId: z.string().min(1),
  records: z.array(jobsRecordSchema).min(1),
});

export type JobsEvent = z.infer<typeof jobsEventSchema>;
