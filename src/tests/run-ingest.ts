import { ingestMails } from "@/lib/ingestMails";
import { isoNowIST } from "@/lib/isoNowIST";

const res = await ingestMails({
    query: "label:jobs-mails-applied-jobs",
    mode: "Sync",
});

console.log(`${isoNowIST()}\t [ingestMails:Log]\t res ${JSON.stringify(res, null, 2)}`);
