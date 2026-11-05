import { describe, expect, test } from "bun:test";
import { JOBS_TOPICS } from "@/lib/jobs-subscriber/topics";

describe("jobs-subscriber topics", () => {
  test("active topics are non-empty strings from env", () => {
    expect(JOBS_TOPICS.ingestWriteModel.length).toBeGreaterThan(0);
    expect(JOBS_TOPICS.ingestReadModel.length).toBeGreaterThan(0);
    expect(JOBS_TOPICS.status.length).toBeGreaterThan(0);
  });

  test("DLQ topics are distinct from active topics", () => {
    expect(JOBS_TOPICS.ingestWriteModelDlq).not.toBe(JOBS_TOPICS.ingestWriteModel);
    expect(JOBS_TOPICS.ingestReadModelDlq).not.toBe(JOBS_TOPICS.ingestReadModel);
  });

  test("no status DLQ is configured as a subscriber topic constant", () => {
    const keys = Object.keys(JOBS_TOPICS);
    expect(keys.some((k) => k.toLowerCase().includes("status") && k.toLowerCase().includes("dlq"))).toBe(false);
  });
});
