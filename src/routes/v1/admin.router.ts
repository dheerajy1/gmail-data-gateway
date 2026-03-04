import { Elysia } from "elysia";
import bulkExtractMails  from "@/routes/v1/admin/extractMails";

const router = new Elysia({prefix: "admin"})
    .use(bulkExtractMails);

export default router;