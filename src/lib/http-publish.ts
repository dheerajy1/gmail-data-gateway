import { env } from "@/lib/env";

const API_KEY = env.KAFKA_API_KEY;
const API_KEY_SECRET = env.KAFKA_API_KEY_SECRET;
const TOPIC = env.KAFKA_TOPIC;
const JOB_NAME = env.SQL_JOB_NAME;

/**
* Pokes the On-Prem server to start the SQL Agent sync job.
*/
export async function sendHttpPublish() {
    try {
        const resultRaw = await fetch(`${env.ONPREM_SERVER_URL}/api/v1/http-publish`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                "x-client-id": env.CLIENT_ID,
                "x-client-secret": env.CLIENT_SECRET,
                "x-api-key": API_KEY,
                "x-api-secret": API_KEY_SECRET,
            },
            body: JSON.stringify({
                topic: TOPIC,
                jobName: JOB_NAME,
            }),
        });

        if (!resultRaw.ok) {
            throw new Error(`HTTP error! status: ${resultRaw.status}`);
        }

    } catch (error: unknown) {
        console.error(`postLogs https end point`, error);
        // Handle or rethrow the error here
    }
}