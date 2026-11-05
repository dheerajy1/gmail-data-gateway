import z from "zod";

//  response schemas (lightweight)
const AppHealthDataSchema = z.object({
  status: z.literal("UP"),
  version: z.string(),
  uptimeMs: z.number(),
  timestampUtc: z.string(),
  systemMemoryMetrics: z.object({
    rssMb: z.number(),
    heapTotalMb: z.number(),
    heapUsedMb: z.number(),
    externalMb: z.number(),
    arrayBuffersMb: z.number(),
    heapUsagePercent: z.number(),
  }),
});

export type AppHealthDataSchemaType = z.infer<typeof AppHealthDataSchema>;

const DbHealthDataSchema = z.object({
  status: z.enum(["UP", "DOWN"]),
  database: z.enum(["CONNECTED", "CONNECTING", "DISCONNECTED"]),
  latencyMs: z.number().nullable(),
});

export type DbHealthDataSchemaType = z.infer<typeof DbHealthDataSchema>;

const UnifiedHealthDataSchema = z.object({
  app: AppHealthDataSchema,
  db: DbHealthDataSchema,
});

export const SuccessAppSchema = z.object({
  success: z.literal(true),
  httpCode: z.literal(200),
  message: z.string(),
  data: AppHealthDataSchema,
});

export const SuccessDbSchema = z.object({
  success: z.literal(true),
  httpCode: z.literal(200),
  message: z.string(),
  data: DbHealthDataSchema,
});

export const SuccessUnifiedSchema = z.object({
  success: z.literal(true),
  httpCode: z.literal(200),
  message: z.string(),
  data: UnifiedHealthDataSchema,
});
