/**
 * Jobs Kafka topics — values from validated environment configuration.
 */

import { env } from "@/lib/env";

export const JOBS_TOPICS = {
  get ingestWriteModel(): string {
    return env.JOBS_WRITE_TOPIC;
  },
  get ingestWriteModelDlq(): string {
    return env.JOBS_WRITE_DLQ_TOPIC;
  },
  get ingestReadModel(): string {
    return env.JOBS_READ_TOPIC;
  },
  get ingestReadModelDlq(): string {
    return env.JOBS_READ_DLQ_TOPIC;
  },
  get status(): string {
    return env.JOBS_STATUS_TOPIC;
  },
};

export type JobsTopic = string;
