import { env } from "@/lib/env";
import { apiResponses } from "@/lib/errors";
import { ingestMails } from "@/lib/ingestMails";
import { JOBS_TOPICS } from "@/lib/jobs-subscriber/topics";
import { postToGateway } from "@/lib/post-to-gateway";
import { Auth } from "@/middleware/auth";
import { AuthHeadersSchema } from "@/schemas/auth.schema";
import { jobsEventSchema } from "@/schemas/jobs-event.schema";
import { randomUUIDv7 } from "bun";
import { Elysia } from "elysia";
import z from "zod";

const router = new Elysia().use(Auth);

router.post(
  "/extractMails",
  async ({ body, status }) => {
    const { type, query, inputMailsCount, inputDateAfter, inputDateBefore } =
      body;

    const { data: dataEm } = await ingestMails({
      query,
      mode: type,
      inputMailsCount,
      inputDateAfter,
      inputDateBefore,
    });

    if (dataEm.length === 0) {
      return status(200, {
        success: true,
        httpCode: 200,
        message: "No messages found for the given query.",
        data: {
          correlationId: null,
          recordCount: 0,
          status: "NO_NEW_RECORDS",
        },
      });
    }

    const records = dataEm.map((m) => ({
      id: String(m.id),
      date: String(m.date),
      from: String(m.from),
      subject: String(m.subject),
      snippet: String(m.snippet ?? ""),
    }));

    const correlationId = randomUUIDv7();

    const event = jobsEventSchema.parse({
      correlationId,
      job: {
        writeModel: env.JOBS_WRITE_MODEL_PROCEDURE,
        readModel: env.JOBS_READ_MODEL_PROCEDURE,
      },
      sourceClientId: env.SOURCE_CLIENT_ID,
      records,
    });

    // Publish write-model event only — do NOT call Azure SP from the HTTP route.
    await postToGateway({
      topic: JOBS_TOPICS.ingestWriteModel,
      correlationId,
      key: correlationId,
      value: event as unknown as Record<string, unknown>,
    });

    return status(200, {
      success: true,
      httpCode: 200,
      message: `${type} accepted. Write-model event published; Azure and PostgreSQL processing is asynchronous.`,
      data: {
        correlationId,
        recordCount: records.length,
        status: "ACCEPTED",
      },
    });
  },
  {
    Auth: true,
    body: z
      .object({
        type: z.enum(["Bulk", "Increment", "Sync"]),
        query: z.string().min(1),
        inputMailsCount: z.number().optional(),
        inputDateAfter: z.string().optional(),
        inputDateBefore: z.string().optional(),
      })
      .superRefine((data, ctx) => {
        if (data.type === "Bulk") {
          if (data.inputDateAfter || data.inputDateBefore) {
            ctx.addIssue({
              code: "custom",
              message: "Bulk mode does not allow date filters",
            });
          }
        }

        if (data.type === "Sync") {
          if (data.inputDateAfter || data.inputDateBefore) {
            ctx.addIssue({
              code: "custom",
              message: "Sync mode uses DB date. Do not send manual dates.",
            });
          }
        }

        if (data.type === "Increment") {
          if (!data.inputDateAfter && !data.inputDateBefore) {
            ctx.addIssue({
              code: "custom",
              message:
                "Increment mode requires inputDateAfter or inputDateBefore",
            });
          }
        }
      }),
    headers: AuthHeadersSchema,
    detail: {
      tags: ["admin"],
      summary: "Extract mails and publish Jobs write-model event",
      description:
        "Authenticated user triggers Gmail ingest and publishes a Kafka write-model event. Azure insert and PostgreSQL read-model run asynchronously in the Jobs subscriber.",
      responses: {
        ...apiResponses({
          success: {
            200: z.object({
              success: z.literal(true),
              httpCode: z.literal(200),
              message: z.string(),
              data: z.object({
                correlationId: z.string().nullable(),
                recordCount: z.number(),
                status: z.string(),
              }),
            }),
          },
          error: [
            {
              code: "UNAUTHORIZED",
              subKey: [
                "MISSING_AUTH_HEADERS",
                "USER_NOT_FOUND",
                "INVALID_CREDENTIALS",
              ],
            },
            {
              code: "BAD_REQUEST",
              subKey: [
                "EMPTY_PAYLOAD",
                "INVALID_JSON_INPUT",
                "MALFORMED_INPUT_ROWS",
              ],
            },
            {
              code: "NOT_FOUND",
              subKey: ["TABLE_NOT_FOUND", "NO_EMAIL_DATA"],
            },
            {
              code: "CONFLICT",
              subKey: ["NO_NEW_RECORDS", "DATABASE_CONFLICT"],
            },
            {
              code: "INTERNAL_SERVER_ERROR",
              subKey: ["INGEST_OPERATION_FAILED", "EXTRACT_FAILED"],
            },
          ],
        }),
      },
    },
  },
);

export default router;
