import { sendHttpPublish } from "@/lib/http-publish";

async function main() {
  console.log("Sending HTTP publish...");

  await sendHttpPublish();

  console.log("Done.");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
