import { getGmail } from "@/lib/gmail";
import { sleep } from "@/lib/utlis";
import { gmail_v1 } from "googleapis";
import { DetailedMessage, expandMessages } from "./message-detail";

const PAGE_SIZE = 100;
const PAGE_DELAY_MS = 300;

export async function listAndExpandMessages({
  query,
  maxResults,
}: {
  query: string;
  maxResults?: number;
}): Promise<DetailedMessage[]> {
  const gmail = await getGmail();
  const allMessages: DetailedMessage[] = [];
  let pageToken: string | undefined = undefined;

  do {
    const listRes: gmail_v1.Schema$ListMessagesResponse = (
      await gmail.users.messages.list({
        userId: "me",
        q: query,
        maxResults: PAGE_SIZE,
        pageToken,
      })
    ).data;

    const messagesMetadata = listRes.messages;

    if (!messagesMetadata || messagesMetadata.length === 0) break;

    const ids = messagesMetadata
      .filter((m: gmail_v1.Schema$Message) => !!m.id)
      .map((m: gmail_v1.Schema$Message) => ({ id: m.id! }));

    const detailedData = await expandMessages({ ids });
    allMessages.push(...detailedData);

    pageToken = listRes.nextPageToken ?? undefined;

    if (maxResults !== undefined && allMessages.length >= maxResults) {
      return allMessages.slice(0, maxResults);
    }

    if (pageToken) {
      await sleep(PAGE_DELAY_MS);
    }
  } while (pageToken);

  return allMessages;
}
