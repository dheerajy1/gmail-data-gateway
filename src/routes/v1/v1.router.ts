import adminRouter from "@/routes/v1/admin.router";
import authRouter from "@/routes/v1/auth.router";
import { Elysia } from "elysia";

const router = new Elysia({ prefix: "/api/v1" })
    .use(authRouter)
    .use(adminRouter);

export default router;
