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
