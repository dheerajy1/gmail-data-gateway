import { Elysia } from "elysia";
import extractMails  from "@/routes/v1/admin/extractMails";

const router = new Elysia({prefix: "admin"})
    .use(extractMails);

export default router;
