import { Healthz } from "@/components/Healthz";
import { getAppHealth } from "@/lib/health/get-app-health";
import { getDbHealth } from "@/lib/health/get-db-health";
import { openapi } from "@elysiajs/openapi";
import { Elysia } from "elysia";
import React from "react";
import { renderToReadableStream } from "react-dom/server.browser";

const router = new Elysia({ prefix: "/healthz" }).use(
  openapi({
    documentation: {
      tags: [{ name: "portal-paths" }, { name: "health" }],
    },
  }),
);

router.get(
  "",
  async ({ headers, status }) => {
    const [app, db] = await Promise.all([getAppHealth(), getDbHealth()]);
    const body = {
      success: true,
      httpCode: 200,
      message: "Health check OK",
      data: { app, db },
    };

    const accept = headers.accept ?? "";
    if (!accept.includes("text/html")) {
      return status(200, body);
    }

    const html = React.createElement(Healthz, {
      title: "Health Check | Record log API",
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
