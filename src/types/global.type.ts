import type WebSocket from "ws";
import type { WsPublishTransport } from "@/lib/gateway-ws/ws-publish-transport";

// =========================
// GATEWAY SUBSCRIBER
// =========================

export interface InboundEventMeta {
  headers: Record<string, string>;
  partition: number | null;
  offset: string | null;
}

export interface OpenSubscriberOptions {
  topics: string[];
  logPrefix: string;
  onEvent: (
    topic: string,
    payload: Record<string, unknown>,
    meta: InboundEventMeta,
  ) => Promise<void>;
}

export interface GatewaySubscriberState {
  ws: WebSocket | null;
  transport: WsPublishTransport | null;
  reconnectAttempt: number;
  stopping: boolean;
  connectInFlight: boolean;
  reconnectTimer: ReturnType<typeof setTimeout> | null;
}

export interface SubscriberHandle {
  getTransport: () => WsPublishTransport | null;
  stop: () => void;
}

export interface SubscriberHandlerDeps {
  getTransport: () => ReturnType<SubscriberHandle["getTransport"]>;
}

export type TopicHandler = (
  payload: Record<string, unknown>,
  meta: InboundEventMeta,
  deps: SubscriberHandlerDeps,
) => Promise<void>;

export type WsPublishRequest = {
  topic: string;
  key?: string;
  value: object;
  headers?: Record<string, string>;
};

export type WsLike = {
  send: (data: string) => void;
  readyState: number;
};

export type CustPending = {
  resolve: (meta: {
    topic: string;
    partition: number;
    offset: string;
  }) => void;
  reject: (err: Error) => void;
  timer: ReturnType<typeof setTimeout>;
};

export type FailureContext = {
  transport: WsPublishTransport;
  stageTopic: string;
  dlqTopic: string;
  payload: Record<string, unknown>;
  correlationId: string;
  currentRetryCount: number;
  failureReason: string;
  originalPartition?: number | null;
  originalOffset?: string | null;
  logPrefix: string;
};

export type ApplyStatusResult = {
  result: string;
  replayed: boolean;
  assignedSequence: number;
  currentStage: string;
  currentStatus: string;
};

export type WriteModelSpNotes = {
  rowsInserted: number;
  message?: string;
  replayed?: boolean;
};

export type WriteModelSpResult = {
  success: true;
  data: WriteModelSpNotes;
};

export type ReadModelHandlerDeps = {
  getTransport: () => WsPublishTransport | null;
};

export type StatusHandlerDeps = {
  getTransport: () => WsPublishTransport | null;
};

export type WriteModelHandlerDeps = {
  getTransport: () => WsPublishTransport | null;
};

export type ReadModelApplyResult = {
  result: string;
  replayed: boolean;
  rowsInserted: number | null;
};
