import { getGmail } from "@/lib/gmail";
import { sleep } from "@/lib/utlis";
import { gmail_v1 } from "googleapis";

const PAGE_SIZE = 100;
const QUERY = "label:jobs-mails-applied-jobs";
const PAGE_DELAY_MS = 300;

async function countBulk() {
  const gmail = await getGmail();
  let total = 0;
  let pageToken: string | undefined = undefined;

  do {
    const listRes: gmail_v1.Schema$ListMessagesResponse =
      (await gmail.users.messages.list({
        userId: "me",
        q: QUERY,
        maxResults: PAGE_SIZE,
        pageToken
      })).data;

    const messages = listRes.messages ?? [];
    total += messages.length;
    pageToken = listRes.nextPageToken ?? undefined;

    console.log(`Page fetched: ${messages.length} | Running total: ${total}`);

    if (pageToken) {
      await sleep(PAGE_DELAY_MS);
    }
  } while (pageToken);

  console.log("Final Bulk count:", total);
}

countBulk().catch((err) => {
  console.error(err);
  process.exit(1);
});

process.exit(0);
