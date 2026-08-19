import { isoNowIST } from "@/lib/isoNowIST";
import { inboundEventSchema } from "@/schemas/global.schema";
import {
  GatewaySubscriberState,
  OpenSubscriberOptions,
} from "@/types/global.type";

export async function handleGatewayMessage(
  data: string,
  state: GatewaySubscriberState,
  opts: OpenSubscriberOptions,
): Promise<void> {
  const msg = JSON.parse(data) as Record<string, unknown>;
  const type = msg.type;

  if (type === "published" || (type === "error" && msg.requestId)) {
    state.transport?.handleControlMessage(msg);
    return;
  }

  if (type === "ready") {
    console.log(`${isoNowIST()}\t[${opts.logPrefix}:Control]\tready`);
    return;
  }

  if (type === "subscribed") {
    console.log(
      `${isoNowIST()}\t[${opts.logPrefix}:Control]\tsubscribed topic=${msg.topic}`,
    );
    return;
  }

  if (type === "event") {
    const parsed = inboundEventSchema.safeParse(msg);
    if (!parsed.success) {
      console.error(
        `${isoNowIST()}\t[${opts.logPrefix}:Error]\tinvalid event payload corr=${msg.correlationId ?? "?"}`,
        parsed.error.flatten(),
      );
      return;
    }

    const {
      type: _type,
      topic,
      headers = {},
      partition,
      offset,
      ...payload
    } = parsed.data;

    console.log(
      `${isoNowIST()}\t[${opts.logPrefix}:Event]\ttopic=${topic}\tcorr=${msg.correlationId ?? "?"}\tretry=${headers["x-jobs-retry-count"] ?? "0"}`,
    );

    await opts.onEvent(topic, payload, { headers, partition, offset });
    return;
  }

  if (type === "error") {
    console.error(
      `${isoNowIST()}\t[${opts.logPrefix}:Control]\terror`,
      msg.message ?? msg,
    );
    return;
  }

  console.log(
    `${isoNowIST()}\t[${opts.logPrefix}:Control]\t${type ?? "unknown"}`,
    data.slice(0, 200),
  );
}
