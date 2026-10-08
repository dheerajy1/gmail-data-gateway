import adminRouter from "@/routes/v1/admin.router";
import authRouter from "@/routes/v1/auth.router";
import healthRouter from "@/routes/v1/health.router";
import { Elysia } from "elysia";

const router = new Elysia({ prefix: "/api/v1" })
    .use(healthRouter)
    .use(authRouter)
    .use(adminRouter);

export default router;
