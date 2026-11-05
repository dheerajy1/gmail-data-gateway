/**
 * Permanent Jobs subscriber.
 *
 * ONE process · ONE authenticated WS · THREE consumed topics:
 *   jobs-ingest-write-model → Azure write-model (+ retry / DLQ)
 *   jobs-ingest-read-model  → PostgreSQL read-model (+ retry / DLQ)
 *   jobs-status             → fn_pipeline_apply_status
 */

import { openGatewaySubscriber } from "@/lib/gateway-ws/open-gateway-subscriber";
import { isoNowIST } from "@/lib/isoNowIST";
import { handleReadModelEvent } from "@/lib/jobs-subscriber/handlers/read-model";
import { handleStatusEvent } from "@/lib/jobs-subscriber/handlers/status";
import { handleWriteModelEvent } from "@/lib/jobs-subscriber/handlers/write-model";
import { JOBS_TOPICS } from "@/lib/jobs-subscriber/topics";
import type {
  InboundEventMeta,
  SubscriberHandle,
  TopicHandler,
} from "@/types/global.type";

const SUBSCRIBED_TOPICS = [
  JOBS_TOPICS.ingestWriteModel,
  JOBS_TOPICS.ingestReadModel,
  JOBS_TOPICS.status,
] as const;

const TOPIC_HANDLERS: Record<string, TopicHandler> = {
  [JOBS_TOPICS.ingestWriteModel]: handleWriteModelEvent,
  [JOBS_TOPICS.ingestReadModel]: handleReadModelEvent,
  [JOBS_TOPICS.status]: handleStatusEvent,
};

let handle: SubscriberHandle | null = null;

async function onEvent(
  topic: string,
  payload: Record<string, unknown>,
  meta: InboundEventMeta,
): Promise<void> {
  const handler = TOPIC_HANDLERS[topic];

  if (!handler) {
    console.log(
      `${isoNowIST()}\t[JobsSub]\tignore unknown topic=${topic} corr=${payload.correlationId ?? "?"}`,
    );
    return;
  }

  await handler(payload, meta, {
    getTransport: () => handle?.getTransport() ?? null,
  });
}

export function startJobsSubscriber(): void {
  console.log(
    `${isoNowIST()}\t[JobsSub:Start]\ttopics=${SUBSCRIBED_TOPICS.join(",")}`,
  );
  handle = openGatewaySubscriber({
    topics: [...SUBSCRIBED_TOPICS],
    logPrefix: "JobsSub",
    onEvent,
  });
}

export function stopJobsSubscriber(): void {
  handle?.stop();
  handle = null;
}

if (import.meta.main) {
  startJobsSubscriber();

  const shutdown = (signal: string) => {
    console.log(`${isoNowIST()}\t[JobsSub:Shutdown]\t${signal}`);
    stopJobsSubscriber();
    process.exit(0);
  };

  process.on("SIGINT", () => shutdown("SIGINT"));
  process.on("SIGTERM", () => shutdown("SIGTERM"));
}
