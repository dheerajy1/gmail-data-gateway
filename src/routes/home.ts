import { Home } from "@/components/Home";
import { Elysia } from "elysia";
import React from "react";
import { renderToReadableStream } from "react-dom/server.browser";

const router = new Elysia({ prefix: "/" })

router.get("", async () => {
    const html = React.createElement(Home, { title: `Elysia + Bun ${process.versions.bun} on Vercel`, bunVersion: process.versions.bun });

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
