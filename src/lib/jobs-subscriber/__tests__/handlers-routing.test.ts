import { describe, expect, test } from "bun:test";
import { JOBS_TOPICS } from "@/lib/jobs-subscriber/topics";
import { shouldDlq, MAX_RETRIES, getRetryCount, RETRY_COUNT_HEADER } from "@/lib/jobs-subscriber/retry-policy";
import { jobsEventSchema } from "@/schemas/jobs-event.schema";
import { pipelineStatusEventSchema } from "@/schemas/pipeline-status.schema";

describe("jobs-subscriber routing constants", () => {
  test("three active topics are distinct", () => {
    const active = [
      JOBS_TOPICS.ingestWriteModel,
      JOBS_TOPICS.ingestReadModel,
      JOBS_TOPICS.status,
    ];
    expect(new Set(active).size).toBe(3);
  });

  test("DLQ topics exist only for write and read", () => {
    expect(JOBS_TOPICS.ingestWriteModelDlq).toContain("dlq");
    expect(JOBS_TOPICS.ingestReadModelDlq).toContain("dlq");
    expect(JOBS_TOPICS.status.toLowerCase()).not.toContain("dlq");
  });
});

describe("jobs event schema", () => {
  test("accepts valid write/read payload", () => {
    const event = jobsEventSchema.parse({
      correlationId: "018f3a2b-4c5d-7e8f-9a0b-1c2d3e4f5a6b",
      job: {
        writeModel: "[02sjobsData].[02spinsertRawAppliedJobs]",
        readModel: "sp_kafka_apply_committed_records",
      },
      sourceClientId: "jobs-api-gateway",
      records: [
        {
          id: "abc123",
          date: "2026-01-01T10:00:00+05:30",
          from: "hr@example.com",
          subject: "Application",
          snippet: "Thanks",
        },
      ],
    });
    expect(event.records).toHaveLength(1);
    expect(event.correlationId).toBe("018f3a2b-4c5d-7e8f-9a0b-1c2d3e4f5a6b");
  });

  test("rejects empty records", () => {
    expect(() =>
      jobsEventSchema.parse({
        correlationId: "018f3a2b-4c5d-7e8f-9a0b-1c2d3e4f5a6b",
        job: { writeModel: "x", readModel: "y" },
        sourceClientId: "jobs-api-gateway",
        records: [],
      }),
    ).toThrow();
  });
});

describe("status event schema", () => {
  test("preserves eventId field", () => {
    const ev = pipelineStatusEventSchema.parse({
      eventId: "018f3a2b-4c5d-7e8f-9a0b-1c2d3e4f5a6b",
      correlationId: "018f3a2b-4c5d-7e8f-9a0b-1c2d3e4f5a6c",
      stage: "RECEIVED",
      status: "SUCCEEDED",
      timestamp: new Date().toISOString(),
      sequence: null,
    });
    expect(ev.eventId).toBe("018f3a2b-4c5d-7e8f-9a0b-1c2d3e4f5a6b");
  });
});

describe("retry policy", () => {
  test("rowsInserted=0 is not a retry concern (policy pure)", () => {
    expect(shouldDlq(0)).toBe(false);
    expect(getRetryCount({ [RETRY_COUNT_HEADER]: "3" })).toBe(3);
    expect(shouldDlq(MAX_RETRIES)).toBe(true);
  });
});
