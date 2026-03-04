import { ALLOWED_ORIGINS, publicPath } from "@/lib/constants";
import { MyError, errors } from "@/lib/errors";
import { serverGate } from "@/middleware/server-gate";
import aboutRouter from "@/routes/about";
import healthzRouter from "@/routes/healthz";
import homeRouter from "@/routes/home";
import loginRouter from "@/routes/login";
import v1Router from "@/routes/v1/v1.router";
import { cors } from "@elysiajs/cors";
import { openapi } from "@elysiajs/openapi";
import staticPlugin from "@elysiajs/static";
import { Elysia } from "elysia";
import { ZodError } from "zod";

// ===============================================
// Elysia
// ===============================================

const app = new Elysia()
  .error({
    MyError,
  })
  .use(
    openapi({
      path: "/openapi",
      documentation: {
        info: {
          title: "Gmail data gateway API",
          version: "v1"
        }
      },
      exclude: {
        paths: [
          "/*",
          "/",
          "/login",
          "/home",
          "/about",
          "/healthz",
        ]
      }
    })
  )
  .onError(({ error, status, code }) => {

    switch (true) {

      case error instanceof MyError:
        return status(error.httpCode, {
          success: false,
          httpCode: error.httpCode,
          code: error.code,
          message: error.message,
          error: error.error,
        });

      case error instanceof ZodError:
        return status(400, {
          success: false,
          httpCode: 400,
          code: error.name,
          message: error.message,
          error: error.issues,
        });

      case code === 'VALIDATION':
        return status(error.status, {
          success: false,
          httpCode: error.status,
          code: error.code,
          message: error.messageValue?.path
            ? Array.isArray(error.messageValue.path)
              ? error.messageValue.path.flat(Infinity).toString()
              : error.messageValue.path
            : '',
          error: error.customError,
        });

      default:
        return status(500, {
          success: false,
          httpCode: 500,
          code: 'INTERNAL_SERVER_ERROR',
          message: errors.INTERNAL_SERVER_ERROR.SERVER_ERROR.message,
          error: errors.INTERNAL_SERVER_ERROR.SERVER_ERROR.error,
        });
    }

  });

// ===============================================
// CORS
// ===============================================

app.use(
  cors({
    origin: (request) => {
      const origin = request.headers.get("origin");

      // allow non-browser / server-to-server
      if (!origin) return true;

      if (ALLOWED_ORIGINS.fixed.includes(origin)) return true;

      if (
        ALLOWED_ORIGINS.wildcards.some((domain) =>
          origin.endsWith(domain)
        )
      )
        return true;

      return false;
    },
    credentials: true,
    allowedHeaders: [
      "Content-Type",
      "Authorization",
      "X-Client-Id",
      "X-Client-Secret"
    ],
    methods: ["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"]
  })
);

// ===============================================
// Static assets
// ===============================================

app.use(staticPlugin({
  assets: publicPath,
  prefix: "/",
  alwaysStatic: false,
}));

// ===============================================
// Docs login (ONLY public thing)
// ===============================================

app.use(loginRouter);

// ===============================================
// GLOBAL SERVER GATE (ABSOLUTE)
// ===============================================

app.onBeforeHandle(({ request, cookie }) => {

  const url = new URL(request.url);
  const path = url.pathname;

  // 1. Allow files with extension (static assets)
  if (/\.(css|js|png|jpg|jpeg|svg|ico|txt)$/.test(path)) {
    return;
  }

  // 2. Internal server-to-server (API key headers)
  if (serverGate({ request })) {
    return;
  }

  // 3. Allow login page itself (GET + POST)
  if (path.startsWith("/login")) {
    return;
  }

  // 4. Logged-in browser
  if (cookie.auth?.value === "ok") {
    return;
  }

  // 5. Not logged in → redirect to login
  return new Response(null, {
    status: 302,
    headers: {
      Location: "/login"
    }
  });
});

// ===============================================
// HTML routes (PROTECTED)
// ===============================================

app.use(homeRouter);
app.use(aboutRouter);
app.use(healthzRouter);

// ===============================================
// v1 routes (PROTECTED)
// ===============================================

app.use(v1Router);

export default app;
