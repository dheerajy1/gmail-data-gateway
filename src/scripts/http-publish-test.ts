import { postToGateway } from "@/lib/post-to-gateway";

async function main() {
  console.log("Sending HTTP publish...");

  await postToGateway({
    topic: "gateway-test",
    correlationId: Bun.randomUUIDv7(),
    value: {
      message: "HTTP publish test",
      timestamp: new Date().toISOString(),
    },
  });

  console.log("Done.");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});