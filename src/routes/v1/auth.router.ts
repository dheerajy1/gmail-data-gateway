import { Elysia } from "elysia";
import login from "@/routes/v1/auth/login"

const router = new Elysia()
    .use(login);

export default router;
