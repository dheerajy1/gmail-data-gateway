/** Max retries on the stage topic before DLQ. */
export const MAX_RETRIES = 3;

/** Kafka header: current retry attempt (stringified non-negative integer). */
export const RETRY_COUNT_HEADER = "x-jobs-retry-count";

/** DLQ metadata headers (string values). */
export const HEADER_ORIGINAL_TOPIC = "x-original-topic";
export const HEADER_ORIGINAL_PARTITION = "x-original-partition";
export const HEADER_ORIGINAL_OFFSET = "x-original-offset";
export const HEADER_CORRELATION_ID = "x-correlation-id";
export const HEADER_FAILURE_REASON = "x-failure-reason";

export function getRetryCount(headers: Record<string, string>): number {
  const raw = headers[RETRY_COUNT_HEADER];
  if (raw == null || raw === "") return 0;
  const n = Number.parseInt(raw, 10);
  if (!Number.isFinite(n) || n < 0) return 0;
  return n;
}

export function shouldDlq(currentRetryCount: number): boolean {
  return currentRetryCount >= MAX_RETRIES;
}
