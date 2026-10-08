import { HealthzDb } from "@/components/HealthzDb";
import { getDbHealth } from "@/lib/health/get-db-health";
import { openapi } from "@elysiajs/openapi";
import { Elysia } from "elysia";
import React from "react";
import { renderToReadableStream } from "react-dom/server.browser";

const router = new Elysia({ prefix: "/healthz/db" }).use(
  openapi({
    documentation: {
      tags: [{ name: "portal-paths" }, { name: "health" }],
    },
  }),
);

router.get(
  "",
  async ({ headers, status }) => {
    const data = await getDbHealth();
    const body = {
      success: true,
      httpCode: 200,
      message:
        data.status === "UP"
          ? "Database connection successful"
          : "Database connection failed",
      data,
    };

    const accept = headers.accept ?? "";
    if (!accept.includes("text/html")) {
      return status(200, body);
    }

    const html = React.createElement(HealthzDb, {
      title: "DB Health | Record log API",
      response: body,
    });
    const stream = await renderToReadableStream(html);
    return new Response(stream, {
      headers: { "Content-Type": "text/html" },
    });
  },
  {
    detail: { tags: ["portal-paths", "health"] },
  },
);

export default router;
