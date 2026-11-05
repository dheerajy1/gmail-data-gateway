import { getGmail } from "@/lib/gmail";
import { sleep } from "@/lib/utlis";

export interface DetailedMessage {
  id: string;
  date: string;
  from: string;
  subject: string;
  snippet: string;
}

function formatToISTISO({ dateString }: { dateString: string }) {
  if (!dateString) return null;

  const date = new Date(dateString);
  const utc = date.getTime();
  const istOffsetMs = (5 * 60 + 30) * 60 * 1000;
  const istDate = new Date(utc + istOffsetMs);

  const pad = (n: number) => String(n).padStart(2, "0");

  const YYYY = istDate.getUTCFullYear();
  const MM = pad(istDate.getUTCMonth() + 1);
  const DD = pad(istDate.getUTCDate());
  const HH = pad(istDate.getUTCHours());
  const mm = pad(istDate.getUTCMinutes());
  const ss = pad(istDate.getUTCSeconds());

  return `${YYYY}-${MM}-${DD}T${HH}:${mm}:${ss}+05:30`;
}

function cleanText({ input }: { input: string }) {
  if (input == null) return "";
  let s = String(input);

  try {
    s = s.replace(/\p{C}/gu, "");
  } catch {
    s = s.replace(
      /[\u0000-\u001F\u007F-\u009F\u00A0\u034F\u200B-\u200F\u2060-\u206F\uFEFF]/g,
      "",
    );
  }

  s = s
    .replace(/\s+/g, " ")
    .replace(/[\u034F\u200B-\u200F\u202A-\u202E\uFEFF]/g, "")
    .trim();

  return s;
}

export async function fetchMessageDetail(
  id: string,
): Promise<DetailedMessage | null> {
  try {
    const gmail = await getGmail();

    const res = await gmail.users.messages.get({
      userId: "me",
      id: id,
      format: "metadata",
      metadataHeaders: ["Date", "From", "Subject"],
    });

    const msg = res.data;
    if (!msg.payload?.headers) return null;

    const getH = (name: string) =>
      cleanText({
        input:
          msg.payload?.headers?.find(
            (h) => h.name?.toLowerCase() === name.toLowerCase(),
          )?.value ?? "",
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
      snippet,
    };
  } catch {
    return null;
  }
}

export async function expandMessages({
  ids,
}: {
  ids: { id: string }[];
}): Promise<DetailedMessage[]> {
  const CHUNK_SIZE = 20;
  const BATCH_DELAY_MS = 200;
  const allResults: DetailedMessage[] = [];

  for (let i = 0; i < ids.length; i += CHUNK_SIZE) {
    const chunk = ids.slice(i, i + CHUNK_SIZE);

    const batch = await Promise.all(
      chunk.map((m) => fetchMessageDetail(m.id!)),
    );

    const validMessages = batch.filter((m): m is DetailedMessage => m !== null);
    allResults.push(...validMessages);

    if (i + CHUNK_SIZE < ids.length) {
      await sleep(BATCH_DELAY_MS);
    }
  }

  return allResults;
}
