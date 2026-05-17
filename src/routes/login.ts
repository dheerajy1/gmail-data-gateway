import { Login } from "@/components/Login";
import { getAzureSQLDbPool } from "@/lib/db";
import { errors } from "@/lib/errors";
import { loginSchema } from "@/schemas/login";
import { Elysia } from "elysia";
import React from "react";
import { renderToReadableStream } from "react-dom/server.browser";
import z from "zod";

const router = new Elysia({ prefix: "/login" });

router.get("", async () => {

    const html = React.createElement(Login, { title: `Login | Gmail data gateway API` });

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

router.post("", async ({ request, body, cookie, status }) => {
    // browser-only POST (FORM submit)
    const accept = request.headers.get("accept") ?? "";
    if (!accept.includes("text/html")) {
        return status(403, { message: "Forbidden" });
    }

    const parsed = loginSchema.safeParse(body);

    if (!parsed.success) {
        return status(400, {
            success: false,
            httpCode: 400,
            message: "Invalid request payload"
        });
    }

    const { username, password } = parsed.data;

    try {
        const pool = await getAzureSQLDbPool({ poolKey: "default" });

        const result = await pool
            .request()
            .input("username", username)
            .query(`
                SELECT passwordHash
                FROM [01tportalUsers]
                WHERE userName = @username;
            `);

        if (result.recordset.length === 0) {
            return status(401, {
                success: false,
                httpCode: 401,
                message: "Invalid credentials"
            });
        }

        const { passwordHash } = result.recordset[0];

        const isValid = await Bun.password.verify(password, passwordHash);

        if (!isValid) {
            return status(401, {
                success: false,
                httpCode: 401,
                message: "Invalid credentials"
            });
        }

        cookie.auth.set({
            value: "ok",
            httpOnly: true,
            sameSite: "strict",
            maxAge: 60 * 60 * 24,
            secure: process.env.NODE_ENV === "production"
        });

        return new Response(null, {
            status: 302,
            headers: {
                Location: "/"
            }
        });

    } catch (err: unknown) {

        const e = err as { code?: string };

        if (["ELOGIN", "ETIMEOUT", "ESOCKET"].includes(e.code ?? "")) {
            return status(errors.SERVICE_UNAVAILABLE.httpCode, {
                success: false,
                httpCode: errors.SERVICE_UNAVAILABLE.httpCode,
                message: errors.SERVICE_UNAVAILABLE.DATABASE_UNAVAILABLE.message,
                error: errors.SERVICE_UNAVAILABLE.DATABASE_UNAVAILABLE.error
            });
        }

        return status(errors.INTERNAL_SERVER_ERROR.httpCode, {
            success: false,
            httpCode: errors.INTERNAL_SERVER_ERROR.httpCode,
            message: errors.INTERNAL_SERVER_ERROR.MESSAGE
        });
    }
}, {
    body: z.object({
        username: z.string(),
        password: z.string().min(6),
    }),
    detail: {
        tags: ["portal-paths"],
        // hide: true,
    }
});

export default router;
