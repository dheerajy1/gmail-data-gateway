import { describe, expect, test } from "bun:test";
import {
  MAX_RETRIES,
  RETRY_COUNT_HEADER,
  getRetryCount,
  shouldDlq,
} from "@/lib/jobs-subscriber/retry-policy";

describe("jobs-subscriber retry policy", () => {
  test("MAX_RETRIES is 3", () => {
    expect(MAX_RETRIES).toBe(3);
  });

  test("getRetryCount defaults to 0", () => {
    expect(getRetryCount({})).toBe(0);
    expect(getRetryCount({ [RETRY_COUNT_HEADER]: "" })).toBe(0);
  });

  test("getRetryCount parses header", () => {
    expect(getRetryCount({ [RETRY_COUNT_HEADER]: "2" })).toBe(2);
  });

  test("shouldDlq at boundary", () => {
    expect(shouldDlq(MAX_RETRIES - 1)).toBe(false);
    expect(shouldDlq(MAX_RETRIES)).toBe(true);
  });
});
