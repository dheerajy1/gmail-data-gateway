/**
 * WebSocket transport for kafka-api-gateway subscribers.
 * - publish via gateway WS `publish` command (requestId-correlated)
 * - optional Kafka headers (retry count / DLQ metadata)
 * - processed ACK for inbound events
 */

import { isoNowIST } from "@/lib/isoNowIST";
import { CustPending, WsLike, WsPublishRequest } from "@/types/global.type";
import { randomUUIDv7 } from "bun";

const OPEN = 1; // WebSocket.OPEN

export class WsPublishTransport {
  private pending = new Map<string, CustPending>();
  private publishTimeoutMs: number;

  constructor(
    private getWs: () => WsLike | null,
    opts?: { publishTimeoutMs?: number },
  ) {
    this.publishTimeoutMs = opts?.publishTimeoutMs ?? 30_000;
  }

  /**
   * Publish via the shared WS connection. Resolves when gateway returns
   * matching `published`, rejects on `error` or timeout or close.
   */
  publish(
    req: WsPublishRequest,
  ): Promise<{ topic: string; partition: number; offset: string }> {
    const ws = this.getWs();
    if (!ws || ws.readyState !== OPEN) {
      return Promise.reject(new Error("WebSocket not open"));
    }

    /** Transport request ID only — not the pipeline correlationId. */
    const requestId = randomUUIDv7();

    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(requestId);
        reject(
          new Error(
            `publish timeout requestId=${requestId} topic=${req.topic}`,
          ),
        );
      }, this.publishTimeoutMs);

      this.pending.set(requestId, {
        resolve,
        reject,
        timer,
      });

      const frame: Record<string, unknown> = {
        type: "publish",
        requestId,
        topic: req.topic,
        key: req.key,
        value: req.value,
      };
      if (req.headers && Object.keys(req.headers).length > 0) {
        frame.headers = req.headers;
      }

      console.log(
        `${isoNowIST()}\t[WsTransport:Publish]\trequestId=${requestId}\ttopic=${req.topic}\tkey=${req.key ?? ""}`,
      );

      try {
        ws.send(JSON.stringify(frame));
      } catch (err) {
        clearTimeout(timer);
        this.pending.delete(requestId);
        reject(err instanceof Error ? err : new Error(String(err)));
      }
    });
  }

  /** Handle control messages from gateway (published / error with requestId). */
  handleControlMessage(msg: Record<string, unknown>): boolean {
    const type = msg.type;
    const requestId = typeof msg.requestId === "string" ? msg.requestId : null;
    if (!requestId) return false;

    const p = this.pending.get(requestId);
    if (!p) return false;

    if (type === "published") {
      clearTimeout(p.timer);
      this.pending.delete(requestId);
      p.resolve({
        topic: typeof msg.topic === "string" ? msg.topic : "",
        partition: typeof msg.partition === "number" ? msg.partition : 0,
        offset:
          typeof msg.offset === "string"
            ? msg.offset
            : String(msg.offset ?? ""),
      });
      return true;
    }

    if (type === "error") {
      clearTimeout(p.timer);
      this.pending.delete(requestId);
      const reason =
        typeof msg.reason === "string"
          ? msg.reason
          : typeof msg.message === "string"
            ? msg.message
            : "publish error";
      p.reject(new Error(reason));
      return true;
    }

    return false;
  }

  /** Reject all in-flight publish promises (e.g. on WS close). */
  rejectAllPending(reason: string): void {
    for (const [id, p] of this.pending.entries()) {
      clearTimeout(p.timer);
      p.reject(new Error(`${reason} requestId=${id}`));
    }
    this.pending.clear();
  }

  pendingCount(): number {
    return this.pending.size;
  }

  sendProcessed(correlationId: string): void {
    const ws = this.getWs();
    if (!ws || ws.readyState !== OPEN) {
      throw new Error("WebSocket not open; cannot send processed ACK");
    }
    ws.send(
      JSON.stringify({
        type: "processed",
        correlationId,
      }),
    );
    console.log(
      `${isoNowIST()}\t[WsTransport:ACK]\tprocessed correlationId=${correlationId}`,
    );
  }
}
