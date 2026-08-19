import { env } from "@/lib/env";
import { handleGatewayMessage } from "@/lib/gateway-ws/handle-gateway-message";
import { RECONNECT_BASE_MS, RECONNECT_MAX_MS } from "@/lib/gateway-ws/state";
import { WsPublishTransport } from "@/lib/gateway-ws/ws-publish-transport";
import { isoNowIST } from "@/lib/isoNowIST";
import {
  GatewaySubscriberState,
  OpenSubscriberOptions,
  SubscriberHandle,
} from "@/types/global.type";
import WebSocket, { type RawData } from "ws";

export function scheduleReconnect(
  state: GatewaySubscriberState,
  opts: OpenSubscriberOptions,
): void {
  if (state.stopping || state.connectInFlight) return;

  const delay = Math.min(
    RECONNECT_MAX_MS,
    RECONNECT_BASE_MS * Math.pow(2, Math.min(state.reconnectAttempt, 4)),
  );

  state.reconnectAttempt += 1;

  console.log(
    `${isoNowIST()}\t[${opts.logPrefix}:Reconnect]\tattempt=${state.reconnectAttempt}\tin=${delay}ms`,
  );

  state.reconnectTimer = setTimeout(() => {
    connectGatewaySubscriber(state, opts);
  }, delay);
}

export function attachSocketEvents(
  state: GatewaySubscriberState,
  opts: OpenSubscriberOptions,
): void {
  const ws = state.ws;
  if (!ws) return;

  ws.on("open", () => {
    state.connectInFlight = false;
    state.reconnectAttempt = 0;
    console.log(`${isoNowIST()}\t[${opts.logPrefix}:Open]\tWS connected`);

    for (const topic of opts.topics) {
      ws.send(JSON.stringify({ type: "subscribe", topic }));
    }
  });

  ws.on("message", (data: RawData) => {
    void handleGatewayMessage(data.toString(), state, opts).catch((err) => {
      console.error(
        `${isoNowIST()}\t[${opts.logPrefix}:Error]\tmessage handler`,
        err,
      );
    });
  });

  ws.on("close", () => {
    state.connectInFlight = false;
    console.log(`${isoNowIST()}\t[${opts.logPrefix}:Close]\tWS closed`);
    state.transport?.rejectAllPending("WebSocket closed");
    state.ws = null;
    state.transport = null;
    if (!state.stopping) scheduleReconnect(state, opts);
  });

  ws.on("error", (err: Error) => {
    console.error(`${isoNowIST()}\t[${opts.logPrefix}:Error]\tWS error`, err);
  });
}

export function connectGatewaySubscriber(
  state: GatewaySubscriberState,
  opts: OpenSubscriberOptions,
): void {
  if (state.stopping || state.connectInFlight) return;
  state.connectInFlight = true;

  const url = `${env.ONPREM_SERVER_URL.replace(/^http/, "ws")}/api/v1/ws-subscribe`;
  const ws = new WebSocket(url, {
    headers: {
      "x-client-id": env.CLIENT_ID,
      "x-client-secret": env.CLIENT_SECRET,
      "x-api-key": env.KAFKA_API_KEY,
      "x-api-secret": env.KAFKA_API_KEY_SECRET,
      "x-subscriber-id": env.SOURCE_CLIENT_ID,
    },
  });

  console.log(`${isoNowIST()}\t[${opts.logPrefix}:Connect]\t${url}`);

  state.ws = ws;
  state.transport = new WsPublishTransport(() => state.ws);

  attachSocketEvents(state, opts);
}

export function stopGatewaySubscriber(state: GatewaySubscriberState): void {
  state.stopping = true;
  if (state.reconnectTimer) {
    clearTimeout(state.reconnectTimer);
    state.reconnectTimer = null;
  }
  state.transport?.rejectAllPending("subscriber stopping");
  try {
    state.ws?.close();
  } catch {
    /* ignore close errors while stopping */
  }
  state.ws = null;
  state.transport = null;
}

export function openGatewaySubscriber(
  opts: OpenSubscriberOptions,
): SubscriberHandle {
  if (!opts.topics.length) {
    throw new Error("openGatewaySubscriber requires at least one topic");
  }

  const state: GatewaySubscriberState = {
    ws: null,
    transport: null,
    reconnectAttempt: 0,
    stopping: false,
    connectInFlight: false,
    reconnectTimer: null,
  };

  connectGatewaySubscriber(state, opts);

  return {
    getTransport: () => state.transport,
    stop: () => stopGatewaySubscriber(state),
  };
}
