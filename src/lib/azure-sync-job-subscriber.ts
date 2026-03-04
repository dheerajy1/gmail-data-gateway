import { azureSyncJob } from "@/lib/db-scripts/job-azure-sync";
import { env } from "@/lib/env";
import { isoNowIST } from "@/lib/isoNowIST";
import WebSocket from "ws";

type KafkaMsgVal = {
  correlationId: `${string}-${string}-${string}-${string}-${string}`;
  apiUserId: number;
  jobName: string;
  timestamp: string;
}

const GATEWAY_WS_URL = `${env.ONPREM_SERVER_URL.replace(/^https/, 'wss')}/api/v1/ws-subscribe`;
const API_KEY = env.KAFKA_API_KEY;
const API_KEY_SECRET = env.KAFKA_API_KEY_SECRET;
const TOPIC = env.KAFKA_TOPIC;

const ws = new WebSocket(GATEWAY_WS_URL, {
  headers: {
    "x-client-id": env.CLIENT_ID,
    "x-client-secret": env.CLIENT_SECRET,
    "x-api-key": API_KEY,
    "x-api-secret": API_KEY_SECRET,
  },
});

ws.on("open", () => {
  console.log(`${isoNowIST()}\t [Subscriber:Start]\t WS connected`);

  ws.send(
    JSON.stringify({
      type: "subscribe",
      topic: TOPIC,
    })
  );
});

ws.on("message", async (data) => {
  const { type, topic, correlationId, jobName } = JSON.parse(data.toString()) as unknown as { type: string; topic: string } & KafkaMsgVal;

  // Kafka event forwarded by gateway
  if (type === "event") {

    console.log(`${isoNowIST()}\t [Subscriber:Action]\t EVENT RECEIVED Topic: ${topic}`);

    try {
      await azureSyncJob({ jobName });

      ws.send(
        JSON.stringify({
          type: "processed",
          correlationId,
        })
      );


    } catch (err: unknown) {
      console.error(`${isoNowIST()}\t [Subscriber:Error]\t Failed to trigger SQL Agent job JobName: ${jobName}, Error: `, err);
      // throw err;
    }

  } else {
    console.log(`${isoNowIST()}\t [Subscriber:Log]\t CONTROL`, data);
  }
});

ws.on("close", () => {
  console.log(`${isoNowIST()}\t [Subscriber:Log]\t WS closed`);

});

ws.on("error", (err) => {
  console.log(`${isoNowIST()}\t [Subscriber:Error]\t WS error`, err);
});
