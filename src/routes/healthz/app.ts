import { HealthzApp } from "@/components/HealthzApp";
import { getAppHealth } from "@/lib/health/get-app-health";
import { openapi } from "@elysiajs/openapi";
import { Elysia } from "elysia";
import React from "react";
import { renderToReadableStream } from "react-dom/server.browser";

const router = new Elysia({ prefix: "/healthz/app" }).use(
  openapi({
    documentation: {
      tags: [{ name: "portal-paths" }, { name: "health" }],
    },
  }),
);

router.get(
  "",
  async ({ headers, status }) => {
    const data = getAppHealth();
    const body = {
      success: true,
      httpCode: 200,
      message: "Application is running",
      data,
    };

    const accept = headers.accept ?? "";
    if (!accept.includes("text/html")) {
      return status(200, body);
    }

    const html = React.createElement(HealthzApp, {
      title: "App Health | Record log API",
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
