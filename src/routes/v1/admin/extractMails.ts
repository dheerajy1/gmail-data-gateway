import { spInsertRawDataEmails } from "@/lib/db-scripts/sp-Insert-raw-data-emails";
import { MyError, apiResponses, errors } from "@/lib/errors";
import { sendHttpPublish } from "@/lib/http-publish";
import { ingestMails } from "@/lib/ingestMails";
import { Auth } from "@/middleware/auth";
import { AuthHeadersSchema } from "@/types/auth";
import { Elysia } from "elysia";
import z from "zod";

const router = new Elysia()
    .use(Auth);

router.post(
    "/extractMails",
    async ({ body, status }) => {
        const { type, query, inputMailsCount, inputDateAfter, inputDateBefore } = body;

        const { data: dataEm, error: errorEm } = await ingestMails({
            query,
            mode: type,
            inputMailsCount,
            inputDateAfter,
            inputDateBefore
        });

        if (errorEm) {
            switch (errorEm.number) {

                case 50001:
                    throw new MyError({
                        code: "NOT_FOUND",
                        message: errors.NOT_FOUND.TABLE_NOT_FOUND.message,
                        error: errors.NOT_FOUND.TABLE_NOT_FOUND.error,
                    });

                case 50002:
                    throw new MyError({
                        code: "NOT_FOUND",
                        message: errors.NOT_FOUND.NO_EMAIL_DATA.message,
                        error: errors.NOT_FOUND.NO_EMAIL_DATA.error,
                    });

                default:
                    throw new MyError({
                        code: "INTERNAL_SERVER_ERROR",
                        message: errors.INTERNAL_SERVER_ERROR.INGEST_OPERATION_FAILED.message,
                        error: errors.INTERNAL_SERVER_ERROR.INGEST_OPERATION_FAILED.error,
                    });
            }
        }

        if (!dataEm) {
            throw new MyError({
                code: "INTERNAL_SERVER_ERROR",
                message: errors.INTERNAL_SERVER_ERROR.EXTRACT_FAILED.message,
                error: errors.INTERNAL_SERVER_ERROR.EXTRACT_FAILED.error,
            });
        }

        if (dataEm.length === 0) {
            // ============================
            // Return response
            // ============================
            return status(200, {
                success: true,
                httpCode: 200,
                message: "No messages found for the given query.",
                data: "NO_NEW_RECORDS",
            });
        }

        const { data, error } = await spInsertRawDataEmails({ data: dataEm });

        if (error) {

            switch (error.number) {

                case 50004:
                    throw new MyError({
                        code: "CONFLICT",
                        message: errors.CONFLICT.NO_NEW_RECORDS.message,
                        error: errors.CONFLICT.NO_NEW_RECORDS.error,
                    });

                case 50002:
                    throw new MyError({
                        code: "BAD_REQUEST",
                        message: errors.BAD_REQUEST.EMPTY_PAYLOAD.message,
                        error: errors.BAD_REQUEST.EMPTY_PAYLOAD.error,
                    });

                case 50001:
                    throw new MyError({
                        code: "BAD_REQUEST",
                        message: errors.BAD_REQUEST.INVALID_JSON_INPUT.message,
                        error: errors.BAD_REQUEST.INVALID_JSON_INPUT.error,
                    });

                case 50003:
                    throw new MyError({
                        code: "BAD_REQUEST",
                        message: errors.BAD_REQUEST.MALFORMED_INPUT_ROWS.message,
                        error: errors.BAD_REQUEST.MALFORMED_INPUT_ROWS.error,
                    });

                default:
                    throw new MyError({
                        code: "CONFLICT",
                        message: errors.CONFLICT.DATABASE_CONFLICT.message,
                        error: errors.CONFLICT.DATABASE_CONFLICT.error,
                    });
            }
        }

        if (!data) {
            throw new MyError({
                code: "CONFLICT",
                message: errors.CONFLICT.DATABASE_CONFLICT.message,
                error: errors.CONFLICT.DATABASE_CONFLICT.error,
            });
        }

        await sendHttpPublish();

        // ============================
        // Return response
        // ============================
        return status(200, {
            success: true,
            httpCode: 200,
            message: `${type} operation completed successfully. Emails have been processed and stored.`,
            data: data,
        });
    },
    {
        Auth: true,
        body: z.object({
            type: z.enum(["Bulk", "Increment", "Sync"]),
            query: z.string().min(1),
            inputMailsCount: z.number().optional(),
            inputDateAfter: z.string().optional(),
            inputDateBefore: z.string().optional(),
        }).superRefine((data, ctx) => {

            if (data.type === "Bulk") {
                if (data.inputDateAfter || data.inputDateBefore) {
                    ctx.addIssue({
                        code: "custom",
                        message: "Bulk mode does not allow date filters"
                    });
                }
            }

            if (data.type === "Sync") {
                if (data.inputDateAfter || data.inputDateBefore) {
                    ctx.addIssue({
                        code: "custom",
                        message: "Sync mode uses DB date. Do not send manual dates."
                    });
                }
            }

            if (data.type === "Increment") {
                if (!data.inputDateAfter && !data.inputDateBefore) {
                    ctx.addIssue({
                        code: "custom",
                        message: "Increment mode requires inputDateAfter or inputDateBefore"
                    });
                }
            }

        }),
        headers: AuthHeadersSchema,
        detail: {
            tags: ["admin"],
            summary: "Send / bulk extract mails",
            description: "Authenticated user sends record logs.",
            responses: {
                ...apiResponses({
                    success: {
                        200: z.object({
                            success: z.literal(true),
                            httpCode: z.literal(200),
                            message: z.string(),
                            data: z.string(),
                        })
                    },
                    error: [
                        { code: 'UNAUTHORIZED', subKey: ['MISSING_AUTH_HEADERS', 'USER_NOT_FOUND', 'INVALID_CREDENTIALS'] },
                        { code: 'BAD_REQUEST', subKey: ['EMPTY_PAYLOAD', 'INVALID_JSON_INPUT', 'MALFORMED_INPUT_ROWS'] },
                        { code: 'NOT_FOUND', subKey: ['TABLE_NOT_FOUND', 'NO_EMAIL_DATA'] },
                        { code: 'CONFLICT', subKey: ['NO_NEW_RECORDS', 'DATABASE_CONFLICT'] },
                        { code: 'INTERNAL_SERVER_ERROR', subKey: ['INGEST_OPERATION_FAILED', 'EXTRACT_FAILED'] }
                    ]
                })

            }
        },
    }
);

export default router;
