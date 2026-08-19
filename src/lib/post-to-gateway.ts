import { env } from "@/lib/env";
import { MyError, errors } from "@/lib/errors";
import { isoNowIST } from "@/lib/isoNowIST";
import {
  gatewayHttpPublishRequestSchema,
  gatewayHttpPublishResponseSchema,
  type GatewayHttpPublishRequest,
  type GatewayHttpPublishResponse,
} from "@/schemas/post-to-gateway.schema";

/**
 * Publish one generic transport envelope to the Kafka API Gateway.
 * Validates the response with Zod; does not accept arbitrary JSON.
 */
export async function postToGateway(
  request: GatewayHttpPublishRequest,
): Promise<GatewayHttpPublishResponse> {
  const body = gatewayHttpPublishRequestSchema.parse(request);
  const url = `${env.ONPREM_SERVER_URL.replace(/\/$/, "")}/api/v1/http-publish`;

  try {
    const response = await fetch(url, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "x-client-id": env.CLIENT_ID,
        "x-client-secret": env.CLIENT_SECRET,
        "x-api-key": env.KAFKA_API_KEY,
        "x-api-secret": env.KAFKA_API_KEY_SECRET,
      },
      body: JSON.stringify(body),
    });

    if (!response.ok) {
      throw new MyError({
        code: "INTERNAL_SERVER_ERROR",
        message: errors.INTERNAL_SERVER_ERROR.DATABASE_ERROR.message,
        error: `Kafka Gateway HTTP publish failed status=${response.status}`,
      });
    }

    const parsed = await response.json().catch(() => {
      throw new MyError({
        code: "INTERNAL_SERVER_ERROR",
        message: errors.INTERNAL_SERVER_ERROR.DATABASE_ERROR.message,
        error: "Kafka Gateway returned non-JSON response",
      });
    });

    const validated = gatewayHttpPublishResponseSchema.safeParse(parsed);
    if (!validated.success) {
      throw new MyError({
        code: "INTERNAL_SERVER_ERROR",
        message: errors.INTERNAL_SERVER_ERROR.DATABASE_ERROR.message,
        error: `Kafka Gateway response failed contract validation: ${validated.error.message}`,
      });
    }

    console.log(
      `${isoNowIST()}\t[Jobs:KafkaPublisher]\taccepted topic=${body.topic} correlationId=${body.correlationId}`,
    );

    return validated.data;
  } catch (err: unknown) {
    if (err instanceof MyError) {
      throw err;
    }

    throw new MyError({
      code: "INTERNAL_SERVER_ERROR",
      message: errors.INTERNAL_SERVER_ERROR.DATABASE_ERROR.message,
      error:
        err instanceof Error ? err.message : "Kafka gateway network failure",
    });
  }
}
