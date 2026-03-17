import { ingestMails } from "@/lib/ingestMails";
import { isoNowIST } from "@/lib/isoNowIST";

const res = await ingestMails({
    query: "label:jobs-mails-applied-jobs after:2026/3/06",
    mode: "Increment",
});

console.log(`${isoNowIST()}\t [ingestMails:Log]\t res ${JSON.stringify(res, null, 2)}`);
