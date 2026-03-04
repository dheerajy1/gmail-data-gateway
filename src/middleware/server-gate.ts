import { env } from "@/lib/env";
import { MyError, errors } from "@/lib/errors";
import { NoAuthHeadersSchema } from "@/types/auth";

type serverGateProps = {
  request: Request;
};

export function serverGate({ request }: serverGateProps): boolean {
  try {

    const accept = request.headers.get("accept") ?? "";
    if (accept.includes("text/html")) {
      return false;
    }

    // MANUALLY BUILD THE OBJECT FROM HEADERS
    const headerData = {
      "x-client-id": request.headers.get("x-client-id"),
      "x-client-secret": request.headers.get("x-client-secret"),
    };

    // ----------------------------
    // Present but invalid → BAD_REQUEST
    // ----------------------------
    const parseResult = NoAuthHeadersSchema.safeParse(headerData);

    if (!parseResult.success) {
      throw new MyError({
        code: "BAD_REQUEST",
        message: errors.BAD_REQUEST.INVALID_HEADER_VALUES.message,
        error: errors.BAD_REQUEST.INVALID_HEADER_VALUES.error,
      });
    }

    const parsed = parseResult.data;

    const valid = parsed["x-client-id"] === env.CLIENT_ID && parsed["x-client-secret"] === env.CLIENT_SECRET;

    return valid;
  } catch {

    // console.error(`serverGate error`, error); // (error: unknown)

    return false;
  }
}
