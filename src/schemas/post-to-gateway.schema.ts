import { z } from "zod";

/** Request body for Kafka API Gateway HTTP publish. */
export const gatewayHttpPublishRequestSchema = z.object({
  topic: z.string().min(1),
  correlationId: z.uuid({ version: "v7" }),
  key: z.string().optional(),
  value: z.record(z.string(), z.unknown()),
  headers: z.record(z.string(), z.string()).optional(),
});

export type GatewayHttpPublishRequest = z.infer<
  typeof gatewayHttpPublishRequestSchema
>;

export const gatewayHttpPublishResponseSchema = z
  .object({
    success: z.boolean().optional(),
    topic: z.string().optional(),
    partition: z.number().optional(),
    offset: z.union([z.string(), z.number()]).optional(),
    correlationId: z.string().optional(),
    message: z.string().optional(),
  })
  .passthrough();

export type GatewayHttpPublishResponse = z.infer<
  typeof gatewayHttpPublishResponseSchema
>;
