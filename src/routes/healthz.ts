import { Healthz } from "@/components/Healthz";
import { getDbHealth } from "@/lib/db-scripts/get-db-health";
import openapi from "@elysiajs/openapi";
import { Elysia } from "elysia";
import React from "react";
import { renderToReadableStream } from "react-dom/server.browser";

const router = new Elysia({ prefix: "/healthz" })
    .use(openapi({
        documentation: {
            tags: [
                { name: "portal-paths" },
                { name: "healthz" }
            ]
        }
    }))

router.get("", async ({ headers, status }) => {

    const dbStatus = await getDbHealth() || "unknown";
    const isoTimestamp = new Date().toISOString();

    const accept = headers.accept ?? "";

    // If client does NOT accept html → return JSON
    if (!accept.includes("text/html")) {

        return status(200, {
            success: true,
            httpCode: 200,
            message: "Health check OK",
            data: {
                app: "running",
                db: dbStatus,
                timestamp: isoTimestamp,
            },
        });
    }

    const html = (
        React.createElement(Healthz, {
            title: "Health Check | Gmail data gateway API",
            response: {
                success: true,
                httpCode: 200,
                message: "Health check OK",
                data: {
                    app: "running",
                    db: dbStatus,
                    timestamp: isoTimestamp,
                },
            },
        })
    )

    const stream = await renderToReadableStream(html);

    return new Response(stream, {
        headers: { "Content-Type": "text/html" }
    });

},
    {
        detail: {
            tags: ["portal-paths"]
        }
    }
);

export default router;
