import z from "zod";

// =========================
// ERROR RESPONSE (GLOBAL)
// =========================

export const ErrorResponseSchema = z.object({
  success: z.literal(false),
  statusCode: z.number(),
  message: z.string(),
  error: z.string(),
  code: z.string(),
});

export type ErrorResponse = z.infer<typeof ErrorResponseSchema>;

export const inboundEventSchema = z
  .object({
    type: z.literal("event"),
    topic: z.string().min(1),
    headers: z
      .record(z.string(), z.unknown())
      .optional()
      .transform((raw) => {
        if (!raw) return {};
        return Object.fromEntries(
          Object.entries(raw).filter(
            (entry): entry is [string, string] => typeof entry[1] === "string",
          ),
        );
      }),
    partition: z.number().nullable().optional().default(null),
    offset: z.coerce.string().nullable().optional().default(null),
  })
  .loose();

export const writeModelSpNotesSchema = z.object({
  rowsInserted: z.number(),
  message: z.string().optional(),
  replayed: z.boolean().optional(),
});

export const readModelRowSchema = z.object({
  result: z.string(),
  replayed: z.boolean().optional(),
  rows_inserted: z.number().nullable().optional(),
});
