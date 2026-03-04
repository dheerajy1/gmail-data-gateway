import { spGetLastDateEmails } from "@/lib/db-scripts/sp-get-last-date-emails";
import { errors } from "@/lib/errors";
import { gmail } from "@/lib/gmail";
import { sleep } from "@/lib/utlis";
import { GaxiosError } from "gaxios";
import { gmail_v1 } from "googleapis";

/* =========================
  Types
========================== */

interface DetailedMessage {
  id: string;
  date: string;
  from: string;
  subject: string;
  snippet: string;
}

type IngestError = {
  message: string;
  number?: number;
};

interface ExtractResponse {
  data?: DetailedMessage[];
  error?: IngestError;
}
/* =========================
  Helpers
========================== */

function formatToISTISO({ dateString }: { dateString: string }) {
  if (!dateString) return null;

  const date = new Date(dateString);

  // Get UTC time in ms
  const utc = date.getTime();

  // Add IST offset (5h30m)
  const istOffsetMs = (5 * 60 + 30) * 60 * 1000;
  const istDate = new Date(utc + istOffsetMs);

  const pad = (n: number) => String(n).padStart(2, "0");

  const YYYY = istDate.getUTCFullYear();
  const MM = pad(istDate.getUTCMonth() + 1);
  const DD = pad(istDate.getUTCDate());
  const HH = pad(istDate.getUTCHours());
  const mm = pad(istDate.getUTCMinutes());
  const ss = pad(istDate.getUTCSeconds());

  // return proper ISO with "T" separator
  return `${YYYY}-${MM}-${DD}T${HH}:${mm}:${ss}+05:30`;
}

function cleanText({ input }: { input: string }) {
  if (input == null) return "";
  let s = String(input);

  // Primary: remove all Unicode "Other" characters (controls, format, surrogate, etc.)
  // This requires modern JS engines that support Unicode property escapes.
  try {
    s = s.replace(/\p{C}/gu, "");
  } catch {
    // Fallback for environments without \p support: explicit known ranges + specific codepoints
    s = s.replace(
      /[\u0000-\u001F\u007F-\u009F\u00A0\u034F\u200B-\u200F\u2060-\u206F\uFEFF]/g,
      ""
    );
  }

  // Collapse multiple whitespace (tabs/newlines/spaces) into a single space and trim
  s = s
    .replace(/\s+/g, " ")
    // remove common invisible/control chars (U+034F, zero-width, BOM, etc.)
    .replace(/[\u034F\u200B-\u200F\u202A-\u202E\uFEFF]/g, "")
    .trim();

  return s;
}

async function fetchMessageDetail(id: string): Promise<DetailedMessage | null> {
  try {
    const res = await gmail.users.messages.get({
      userId: "me",
      id: id,
      format: "metadata",
      metadataHeaders: ["Date", "From", "Subject"]
    });

    const msg = res.data;
    if (!msg.payload?.headers) return null;

    const getH = (name: string) =>
      cleanText({
        input: msg.payload?.headers?.find(h => h.name?.toLowerCase() === name.toLowerCase())?.value ?? ""
      });

    const from = getH("From");
    const subject = getH("Subject");
    const snippet = cleanText({ input: msg.snippet ?? "" });
    const dateStr = getH("Date");

    if (!from || !subject || !snippet || !dateStr) return null;

    const formattedDate = formatToISTISO({ dateString: dateStr });
    if (!formattedDate) return null;

    return {
      id: msg.id!,
      date: formattedDate,
      from,
      subject,
      snippet
    };
  } catch {
    // If a single message fails, we skip it rather than crashing the whole batch
    return null;
  }
}

/* =========================
  Main Logic
========================== */

async function expandMessages({ ids }: { ids: { id: string }[] }): Promise<DetailedMessage[]> {
  const CHUNK_SIZE = 20;
  const BATCH_DELAY_MS = 200; // 0.2s pause between batches to respect rate limits
  const allResults: DetailedMessage[] = [];

  for (let i = 0; i < ids.length; i += CHUNK_SIZE) {
    const chunk = ids.slice(i, i + CHUNK_SIZE);

    const batch = await Promise.all(
      chunk.map((m) => fetchMessageDetail(m.id!))
    );

    const validMessages = batch.filter((m): m is DetailedMessage => m !== null);
    allResults.push(...validMessages);

    // Only sleep if there are more chunks to process
    if (i + CHUNK_SIZE < ids.length) {
      await sleep(BATCH_DELAY_MS);
    }
  }

  return allResults;
}

export async function ingestMails({
  query,
  mode,
  inputMailsCount = 100,
  inputDateAfter,
  inputDateBefore
}: {
  query: string;
  mode: "Bulk" | "Increment" | "Sync";
  inputMailsCount?: number;
  inputDateAfter?: string;
  inputDateBefore?: string;
}): Promise<ExtractResponse> {
  try {

    /* =========================
      MODE AND DATE HANDLING
    ========================== */
    let fullQuery = query;
    let count = inputMailsCount;

    if (mode === "Bulk") {
      count = 100; // full paging
    }

    if (mode === "Sync") {
      const res = await spGetLastDateEmails();
      if (res.error) return { error: res.error };

      if (res.data) {
        const unixAfter = Math.floor(new Date(res.data).getTime() / 1000);
        fullQuery = `${query} after:${unixAfter}`;
      }
    }

    if (mode === "Increment") {
      let filters: string[] = [];

      if (inputDateAfter) {
        const unixAfter = Math.floor(new Date(inputDateAfter).getTime() / 1000);
        filters.push(`after:${unixAfter}`);
      }

      if (inputDateBefore) {
        const unixBefore = Math.floor(new Date(inputDateBefore).getTime() / 1000);
        filters.push(`before:${unixBefore}`);
      }

      if (filters.length > 0) {
        fullQuery = `${query} ${filters.join(" ")}`;
      }
    }

    /* =========================
      FETCH LOOP
    ========================== */

    let allMessages: DetailedMessage[] = [];
    let pageToken: string | undefined = undefined;

    do {
      const listRes: gmail_v1.Schema$ListMessagesResponse =
        (await gmail.users.messages.list({
          userId: "me",
          q: fullQuery,
          maxResults: count,
          pageToken
        })).data;

      const messagesMetadata = listRes.messages;

      if (!messagesMetadata || messagesMetadata.length === 0) break;

      const ids = messagesMetadata
        .filter((m: gmail_v1.Schema$Message) => !!m.id)
        .map((m: gmail_v1.Schema$Message) => ({ id: m.id! }));

      const detailedData = await expandMessages({ ids });

      allMessages.push(...detailedData);

      pageToken = listRes.nextPageToken ?? undefined;

      // Stop early if not bulk
      if (mode !== "Bulk" && allMessages.length >= count) break;

    } while (pageToken);

    // FIXME: test
    // console.log(`src/lib/ingestMails.ts allMessages`, allMessages);

    return { data: allMessages };

  } catch (err: unknown) {
    // FIXME: test
    // console.log(`src/lib/ingestMails.ts catch err`, err);

    if (err instanceof GaxiosError) {
      const status = err.response?.status;
      if (status === 401) return { error: { message: errors.UNAUTHORIZED.GMAIL_SESSION_EXPIRED.error } };
      if (status === 403) return { error: { message: errors.FORBIDDEN.GMAIL_ACCESS_DENIED.error } };
      if (status === 429) return { error: { message: errors.TOO_MANY_REQUESTS.GMAIL_RATE_LIMIT.error } };

      return { error: { message: `Gmail API Error (${status}): ${err.message}` } };
    }

    if (typeof err === "object" && err !== null && "number" in err) {
      const sqlErr = err as { message: string; number?: number };
      return {
        error: {
          message: sqlErr.message,
          number: sqlErr.number
        }
      };
    }
    const message =
      err instanceof Error
        ? err.message
        : "An unknown error occurred during ingestion";

    return { error: { message: message } };
  }
}
