import login from "@/routes/v1/auth/login";
import { Elysia } from "elysia";

const router = new Elysia({ prefix: "auth" })
    .use(login);

export default router;
