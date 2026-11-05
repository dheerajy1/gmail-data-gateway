import { spGetLastDateEmails } from "@/lib/db-scripts/sp-get-last-date-emails";
import { MyError, errors } from "@/lib/errors";
import { GaxiosError } from "gaxios";
import { DetailedMessage } from "./message-detail";
import { listAndExpandMessages } from "./fetch-messages";

interface ExtractResponse {
  data: DetailedMessage[];
}

export async function ingestMails({
  query,
  mode,
  inputMailsCount,
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
    let fullQuery = query;
    let maxResults: number | undefined = undefined;

    if (mode === "Bulk") {
      // always full paging – ignore inputMailsCount
      maxResults = undefined;
    }

    if (mode === "Sync") {
      const { data: lastDate } = await spGetLastDateEmails();
      const unixAfter = Math.floor(new Date(lastDate).getTime() / 1000);
      fullQuery = `${query} after:${unixAfter}`;

      // optional hard max – if not given, pull all
      maxResults = inputMailsCount;
    }

    if (mode === "Increment") {
      const filters: string[] = [];

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

      maxResults = inputMailsCount;
    }

    const allMessages = await listAndExpandMessages({
      query: fullQuery,
      maxResults
    });

    return { data: allMessages };
  } catch (err: unknown) {
    if (err instanceof GaxiosError) {
      const status = err.response?.status;

      if (status === undefined) {
        throw new MyError({
          code: "INTERNAL_SERVER_ERROR",
          message: err.message,
          error: err.message
        });
      }

      if ([400, 401].includes(status)) {
        throw new MyError({
          code: "UNAUTHORIZED",
          message: errors.UNAUTHORIZED.GMAIL_SESSION_EXPIRED.message,
          error: errors.UNAUTHORIZED.GMAIL_SESSION_EXPIRED.error
        });
      }

      if (status === 403) {
        throw new MyError({
          code: "FORBIDDEN",
          message: errors.FORBIDDEN.GMAIL_ACCESS_DENIED.message,
          error: errors.FORBIDDEN.GMAIL_ACCESS_DENIED.error
        });
      }

      if (status === 429) {
        throw new MyError({
          code: "TOO_MANY_REQUESTS",
          message: errors.TOO_MANY_REQUESTS.GMAIL_RATE_LIMIT.message,
          error: errors.TOO_MANY_REQUESTS.GMAIL_RATE_LIMIT.error
        });
      }

      throw new MyError({
        code: "INTERNAL_SERVER_ERROR",
        message: `Gmail API Error (${status})`,
        error: err.message
      });
    }

    throw new MyError({
      code: "INTERNAL_SERVER_ERROR",
      message: errors.INTERNAL_SERVER_ERROR.EXTRACT_FAILED.message,
      error: errors.INTERNAL_SERVER_ERROR.EXTRACT_FAILED.error
    });
  }
}
