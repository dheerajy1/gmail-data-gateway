import { apiResponses, errors, MyError } from "@/lib/errors";
import { getAppHealth } from "@/lib/health/get-app-health";
import { getDbHealth } from "@/lib/health/get-db-health";
import { NoAuth } from "@/middleware/auth";
import {
  ErrorResponseSchema,
  NoAuthHeadersSchema,
} from "@/schemas/auth.schema";
import {
  SuccessAppSchema,
  SuccessDbSchema,
  SuccessUnifiedSchema,
} from "@/schemas/health.schema";
import { Elysia } from "elysia";

const router = new Elysia({ prefix: "/health" }).use(NoAuth).model({
  SuccessApp: SuccessAppSchema,
  SuccessDb: SuccessDbSchema,
  SuccessUnified: SuccessUnifiedSchema,
  ErrorResponse: ErrorResponseSchema,
});

// GET /api/v1/health  (unified)
router.get(
  "/",
  async ({ status }) => {
    const [app, db] = await Promise.all([getAppHealth(), getDbHealth()]);

    if (db.status === "DOWN") {
      throw new MyError({
        code: "SERVICE_UNAVAILABLE",
        message: errors.SERVICE_UNAVAILABLE.DB_PROBE_FAILED.message,
        error: errors.SERVICE_UNAVAILABLE.DB_PROBE_FAILED.error,
      });
    }

    return status(200, {
      success: true,
      httpCode: 200,
      message: "Health check OK",
      data: { app, db },
    });
  },
  {
    noAuth: true,
    headers: NoAuthHeadersSchema,
    detail: {
      tags: ["Health"],
      summary: "Unified health check",
      description:
        "Returns application + database health. Requires client credentials.",
      responses: {
        ...apiResponses({
          success: { 200: SuccessUnifiedSchema },
          error: [
            { code: "UNAUTHORIZED", subKey: ["MISSING_AUTH_HEADERS"] },
            { code: "BAD_REQUEST", subKey: ["INVALID_HEADER_VALUES"] },
            { code: "SERVICE_UNAVAILABLE", subKey: ["DB_PROBE_FAILED"] },
          ],
        }),
      },
    },
  },
);

// GET /api/v1/health/app
router.get(
  "/app",
  ({ status }) => {
    const data = getAppHealth();
    return status(200, {
      success: true,
      httpCode: 200,
      message: "Application is running",
      data,
    });
  },
  {
    noAuth: true,
    headers: NoAuthHeadersSchema,
    detail: {
      tags: ["Health"],
      summary: "Application health",
      description: "Returns process uptime, version and memory metrics.",
      responses: {
        ...apiResponses({
          success: { 200: SuccessAppSchema },
          error: [
            { code: "UNAUTHORIZED", subKey: ["MISSING_AUTH_HEADERS"] },
            { code: "BAD_REQUEST", subKey: ["INVALID_HEADER_VALUES"] },
          ],
        }),
      },
    },
  },
);

// GET /api/v1/health/db
router.get(
  "/db",
  async ({ status }) => {
    const data = await getDbHealth();

    if (data.status === "DOWN") {
      throw new MyError({
        code: "SERVICE_UNAVAILABLE",
        message: errors.SERVICE_UNAVAILABLE.DB_PROBE_FAILED.message,
        error: errors.SERVICE_UNAVAILABLE.DB_PROBE_FAILED.error,
      });
    }

    return status(200, {
      success: true,
      httpCode: 200,
      message: "Database connection successful",
      data,
    });
  },
  {
    noAuth: true,
    headers: NoAuthHeadersSchema,
    detail: {
      tags: ["Health"],
      summary: "Database health",
      description: "Probes Azure SQL connectivity and latency.",
      responses: {
        ...apiResponses({
          success: { 200: SuccessDbSchema },
          error: [
            { code: "UNAUTHORIZED", subKey: ["MISSING_AUTH_HEADERS"] },
            { code: "BAD_REQUEST", subKey: ["INVALID_HEADER_VALUES"] },
            { code: "SERVICE_UNAVAILABLE", subKey: ["DB_PROBE_FAILED"] },
          ],
        }),
      },
    },
  },
);

export default router;
