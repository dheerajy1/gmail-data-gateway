import type { OpenAPIV3 } from 'openapi-types';
import z from "zod";

type ErrorCode =
  | "BAD_REQUEST"
  | "UNAUTHORIZED"
  | "PAYMENT_REQUIRED"
  | "FORBIDDEN"
  | "NOT_FOUND"
  | "METHOD_NOT_SUPPORTED"
  | "TIMEOUT"
  | "CONFLICT"
  | "PRECONDITION_FAILED"
  | "PAYLOAD_TOO_LARGE"
  | "UNSUPPORTED_MEDIA_TYPE"
  | "UNPROCESSABLE_CONTENT"
  | "PRECONDITION_REQUIRED"
  | "TOO_MANY_REQUESTS"
  | "CLIENT_CLOSED_REQUEST"
  | "INTERNAL_SERVER_ERROR"
  | "NOT_IMPLEMENTED"
  | "BAD_GATEWAY"
  | "SERVICE_UNAVAILABLE"
  | "GATEWAY_TIMEOUT";

/**
 * ===============================================
 * Error message catalog
 * - Keys are error codes
 * - Values are readonly message maps
 * ===============================================
 */
type ErrorMessageType = {
  readonly [K in ErrorCode]?: Readonly<Record<string, string | number | {
    message: string;
    error: string;
  }>>;
};

/**
 * ===============================================
 * Canonical error messages
 * ===============================================
 */
export const errors = {
  UNAUTHORIZED: {
    httpCode: 401,
    MESSAGE:
      "The client request has not been completed because it lacks valid authentication credentials for the requested resource.",

    DETAIL: "Authorization not provided",

    MISSING_AUTH_HEADERS: {
      message: "Authentication information is required to continue.",
      error: "Missing required authentication headers",
    },

    INVALID_AUTH_HEADERS: {
      message: "The authentication information provided is not valid.",
      error: "Invalid authentication header values",
    },

    MISSING_AUTHORIZATION_TOKEN: {
      message: "You must be logged in to access this resource.",
      error: "Missing required Authorization token",
    },

    EXPIRED_TOKEN: {
      message: "Your session has expired. Please log in again.",
      error: "Expired token",
    },

    INVALID_TOKEN: {
      message: "Your session is no longer valid. Please log in again.",
      error: "Invalid token",
    },

    USER_NOT_FOUND: {
      message: "No account was found with the provided credentials.",
      error: "User does not exist, please signup",
    },

    INVALID_CREDENTIALS: {
      message: "The username or password you entered is incorrect.",
      error: "Invalid credentials",
    },

    MISSING_SUB_OR_ROLE: {
      message: "JWT is missing required subject (sub) or role claim",
      error: "missing_sub_or_role"
    },

    MISSING_API_KEY: {
      message: "Authentication information api key or api secret is required to continue.",
      error: "Missing required authentication api key or api secret headers",
    },

    INVALID_API_KEY: {
      message: "The authentication information api key or api secret provided is not valid.",
      error: "Invalid authentication api key or api secret header values",
    },
    INVALID_API_SECRET: {
      message: "The authentication information api secret provided is not valid.",
      error: "Invalid authentication api secret header values",
    },
    GMAIL_SESSION_EXPIRED: {
      message: "Your Gmail session has expired.",
      error: "AUTH_REQUIRED: Your session has expired."
    }
  },

  BAD_REQUEST: {
    httpCode: 400,
    MESSAGE:
      "The server cannot process the request because the input provided is invalid.",

    DETAIL: "Invalid input data",

    INVALID_HEADER_VALUES: {
      message: "Some request information is invalid or malformed.",
      error: "Invalid authentication header values",
    },

    INVALID_INPUT: {
      message: "One or more input fields contain invalid values.",
      error: "Invalid input data",
    },
    EMPTY_LOGS: {
      message: "At least one log entry is required.",
      error: "Logs array is empty",
    },
    INVALID_LOG_FORMAT: {
      message: "One or more log entries have an invalid format.",
      error: "Invalid log format",
    },
    INVALID_QUERY_FORMAT: {
      message: "One or more query entries have an invalid format.",
      error: "Invalid query format",
    },
    EMPTY_PAYLOAD: {
      message: "Input array was empty.",
      error: "Data is empty",
    },
    INVALID_JSON_INPUT: {
      message:
        "The provided JSON input is invalid or improperly formatted.",
      error:
        "Invalid JSON input format",
    },

    MALFORMED_INPUT_ROWS: {
      message:
        "One or more rows in the input payload are malformed or missing required fields.",
      error:
        "Malformed input rows detected",
    },
  },

  FORBIDDEN: {
    httpCode: 403,
    MESSAGE:
      "You do not have permission to access the requested resource.",

    DETAIL: "Insufficient access",

    ACCESS_DENIED: {
      message: "You are not allowed to perform this action.",
      error: "You do not have permission to perform this action",
    },
    GMAIL_ACCESS_DENIED: {
      message: "Insufficient permissions for Gmail API.",
      error: "ACCESS_DENIED: Check your API permissions."
    }
  },

  NOT_FOUND: {
    httpCode: 404,
    MESSAGE:
      "The requested resource could not be found.",

    DETAIL: "Resource not found",

    RESOURCE_NOT_FOUND: {
      message: "The requested resource does not exist or is no longer available.",
      error: "The requested resource does not exist",
    },
    NO_EMAIL_DATA: {
      message:
        "No email records were found in the system. Please perform a Bulk operation first.",
      error:
        "No email data available for Sync operation",
    },

    TABLE_NOT_FOUND: {
      message:
        "Required email table does not exist in the database.",
      error:
        "Email table not found",
    },
  },

  CONFLICT: {
    httpCode: 409,
    MESSAGE:
      "The request could not be completed due to a conflict with the current state of the resource.",

    DETAIL:
      "A database conflict occurred while processing the request.",

    DATABASE_CONFLICT: {
      message:
        "The operation could not be completed due to a data conflict.",
      error:
        "Database conflict error",
    },

    DUPLICATE_RECORD: {
      message:
        "A record with the same details already exists.",
      error:
        "Duplicate record constraint violation",
    },

    NO_NEW_RECORDS: {
      message:
        "No new records were found to insert.",
      error:
        "No new data available for insertion",
    },

    FOREIGN_KEY_VIOLATION: {
      message:
        "This operation cannot be completed because related data exists.",
      error:
        "Foreign key constraint violation",
    },

    INVALID_STATE: {
      message:
        "The request cannot be processed in the current state of the resource.",
      error:
        "Invalid resource state",
    },

    BUSINESS_RULE_VIOLATION: {
      message:
        "The operation violates a business rule defined in the system.",
      error:
        "Business rule violation",
    },

    CONCURRENCY_CONFLICT: {
      message:
        "The resource was modified by another process. Please retry.",
      error:
        "Concurrency conflict detected",
    },
  },

  PRECONDITION_FAILED: {
    httpCode: 412,
    MESSAGE: "Access to the target resource has been denied.",
    DETAIL: "Precondition failed",

    ETAG_MISMATCH: {
      message: "The provided ETag does not match the current resource.",
      error: "ETag mismatch",
    },
    // new
    IF_MATCH_REQUIRED: {
      message: "The request requires an If-Match header that was not provided.",
      error: "Missing If-Match header",
    },
    MODIFIED_SINCE_FAILED: {
      message: "The resource has been modified since the specified time.",
      error: "Resource modified",
    },
  },

  PAYLOAD_TOO_LARGE: {
    httpCode: 413,
    MESSAGE: "Request entity is larger than limits defined by server.",
    DETAIL: "Payload too large",

    FILE_TOO_LARGE: {
      message: "The uploaded file exceeds the maximum allowed size.",
      error: "File size exceeds limit",
    },
    // new
    BODY_TOO_LARGE: {
      message: "The request body exceeds the server's size limit.",
      error: "Request body too large",
    },
    MULTIPART_TOO_LARGE: {
      message: "One or more parts of the multipart request are too large.",
      error: "Multipart payload exceeds limit",
    },
  },

  UNSUPPORTED_MEDIA_TYPE: {
    httpCode: 415,
    MESSAGE: "The server refuses to accept the request because the payload format is in an unsupported format.",
    DETAIL: "Unsupported media type",

    INVALID_CONTENT_TYPE: {
      message: "The Content-Type header is not supported.",
      error: "Unsupported content type",
    },
    // new
    UNSUPPORTED_CHARSET: {
      message: "The specified charset is not supported.",
      error: "Unsupported charset",
    },
    WRONG_ACCEPT_HEADER: {
      message: "The Accept header specifies an unsupported format.",
      error: "Unsupported accept format",
    },
  },

  UNPROCESSABLE_CONTENT: {
    httpCode: 422,
    MESSAGE: "The server understands the request method, and the request entity is correct, but the server was unable to process it.",
    DETAIL: "Unprocessable entity",

    VALIDATION_FAILED: {
      message: "Validation of the request data failed.",
      error: "Validation error",
    },
    SEMANTIC_ERROR: {
      message: "The request is syntactically correct but semantically invalid.",
      error: "Semantic error",
    },
    // new
    MISSING_DEPENDENCY: {
      message: "Required related data is missing or invalid.",
      error: "Missing dependency",
    },
  },

  PRECONDITION_REQUIRED: {
    httpCode: 428,
    MESSAGE: "The server cannot process the request because a required precondition header is missing.",
    DETAIL: "Precondition required",

    // new
    MISSING_IF_UNMODIFIED_SINCE: {
      message: "The If-Unmodified-Since header is required but missing.",
      error: "Missing If-Unmodified-Since",
    },
    MISSING_IF_NONE_MATCH: {
      message: "The If-None-Match header is required but missing.",
      error: "Missing If-None-Match",
    },
  },

  TOO_MANY_REQUESTS: {
    httpCode: 429,
    MESSAGE: "The rate limit has been exceeded or too many requests are being sent to the server.",
    DETAIL: "Too many requests",

    RATE_LIMIT_EXCEEDED: {
      message: "You have exceeded the allowed number of requests.",
      error: "Rate limit exceeded",
    },
    BURST_LIMIT_EXCEEDED: {
      message: "Too many requests in a short time window.",
      error: "Burst rate limit exceeded",
    },
    DAILY_LIMIT_REACHED: {
      message: "Your daily request quota has been reached.",
      error: "Daily quota exceeded",
    },
    GMAIL_RATE_LIMIT: {
      message: "Gmail API rate limit reached.",
      error: "RATE_LIMIT: Too many requests to Gmail."
    }
  },

  CLIENT_CLOSED_REQUEST: {
    httpCode: 499,
    MESSAGE: "Access to the resource has been denied.",
    DETAIL: "Client closed request",

    // new
    CONNECTION_CLOSED: {
      message: "The client closed the connection before the response was sent.",
      error: "Client connection closed",
    },
    REQUEST_CANCELLED: {
      message: "The client cancelled the request.",
      error: "Request cancelled by client",
    },
  },

  INTERNAL_SERVER_ERROR: {
    httpCode: 500,
    MESSAGE:
      "An unexpected error occurred while processing your request.",
    DETAIL: "Internal server error",

    SERVER_ERROR: {
      message: "Something went wrong. Please try again later.",
      error: "Something went wrong on the server",
    },
    TOKEN_SIGNING_FAILED: {
      message: "Failed to generate authentication token due to an internal error.",
      error: "Token signing failed"
    },
    TOKEN_VERIFICATION_FAILED: {
      message: "Failed to validate authentication token due to an internal error.",
      error: "Token verification failed"
    },
    DATABASE_ERROR: {
      message: "A database error occurred while processing the request.",
      error: "Database error",
    },
    THIRD_PARTY_ERROR: {
      message: "An external service returned an unexpected error.",
      error: "Third-party service error",
    },
    CACHE_FAILURE: {
      message: "Failed to read from or write to the cache layer.",
      error: "Cache operation failed",
    },
    EXTRACT_FAILED: {
      message: "Failed to extract emails from Gmail.",
      error: "Email extraction failed",
    },
    EXTRACT_GMAIL_AUTH: {
      message: "Gmail authentication failed (token missing or invalid).",
      error: "Gmail auth error",
    },
    EXTRACT_GMAIL_API: {
      message: "Gmail API returned an error during fetch.",
      error: "Gmail API failure",
    },
    EXTRACT_DATE_HANDLING: {
      message: "Failed to determine or process email date filter.",
      error: "Date handling error",
    },
    EXTRACT_TOO_MANY_ESTIMATED: {
      message: "Too many emails estimated — limit exceeded.",
      error: "Estimated count too high",
    },
    EXTRACT_NO_EMAILS_FOUND: {
      message: "No emails matched the query and date filter.",
      error: "No matching emails",
    },
    INGEST_OPERATION_FAILED: {
      message:
        "The mail ingestion process failed due to an unexpected server condition.",
      error:
        "Mail ingestion operation failed",
    },
  },

  NOT_IMPLEMENTED: {
    httpCode: 501,
    MESSAGE: "The server does not support the functionality required to fulfill the request.",
    DETAIL: "Not implemented",

    FEATURE_NOT_ENABLED: {
      message: "This feature is not enabled on this server.",
      error: "Feature disabled",
    },
    METHOD_NOT_IMPLEMENTED: {
      message: "This HTTP method is not implemented yet.",
      error: "Method not implemented",
    },
  },

  BAD_GATEWAY: {
    httpCode: 502,
    MESSAGE: "The server received an invalid response from the upstream server.",
    DETAIL: "Bad gateway",

    UPSTREAM_UNAVAILABLE: {
      message: "The upstream service is currently unreachable.",
      error: "Upstream service unavailable",
    },
    INVALID_UPSTREAM_RESPONSE: {
      message: "The upstream server returned an invalid or corrupted response.",
      error: "Invalid upstream response",
    },
  },

  SERVICE_UNAVAILABLE: {
    httpCode: 503,
    MESSAGE: "The server is not ready to handle the request.",
    DETAIL: "Service unavailable",

    MAINTENANCE_MODE: {
      message: "The service is currently under maintenance.",
      error: "Service in maintenance",
    },

    OVERLOADED: {
      message: "The server is temporarily overloaded and cannot handle the request.",
      error: "Server overloaded",
    },
    CIRCUIT_OPEN: {
      message: "Circuit breaker is open due to repeated failures.",
      error: "Circuit breaker open",
    },

    DATABASE_UNAVAILABLE: {
      message: "Database service temporarily unavailable. Please retry later.",
      error: "Database unavailable"
    },
  },

  GATEWAY_TIMEOUT: {
    httpCode: 504,
    MESSAGE: "The server did not get a response in time from the upstream server.",
    DETAIL: "Gateway timeout",

    UPSTREAM_TIMEOUT: {
      message: "The upstream service did not respond within the timeout period.",
      error: "Upstream timeout",
    },
    DNS_RESOLUTION_TIMEOUT: {
      message: "Failed to resolve upstream server address in time.",
      error: "DNS resolution timeout",
    },
  },

  PAYMENT_REQUIRED: {
    httpCode: 402,
    MESSAGE: "The client request requires payment to access the requested resource.",
    DETAIL: "Payment required",
    SUBSCRIPTION_EXPIRED: {
      message: "Your subscription has expired. Please renew to continue.",
      error: "Subscription expired",
    },
  },

  METHOD_NOT_SUPPORTED: {
    httpCode: 405,
    MESSAGE: "The server knows the request method, but the target resource doesn't support this method.",
    DETAIL: "Method not allowed",
    METHOD_NOT_ALLOWED: {
      message: "The HTTP method used is not supported for this endpoint.",
      error: "Method not allowed",
    },
  },

  TIMEOUT: {
    httpCode: 408,
    MESSAGE: "The server would like to shut down this unused connection.",
    DETAIL: "Request timeout",
    REQUEST_TIMEOUT: {
      message: "The request took too long to complete.",
      error: "Request timed out",
    },
  },

} as const satisfies ErrorMessageType;

/**
 * ===============================================
 * Custom error 
 * ===============================================
 */

function getHttpCode(code: ErrorCode): number {
  const entry = errors[code];
  return entry?.httpCode ?? 500;
}

export class MyError extends Error {
  code: ErrorCode;
  error: string;
  httpCode: number;

  constructor(input: {
    code: ErrorCode;
    message: string;
    error: string;
  }) {
    super(input.message);
    Object.setPrototypeOf(this, new.target.prototype);
    this.name = 'MyError';
    this.code = input.code;
    this.error = input.error;
    this.httpCode = getHttpCode(input.code);

    // better stack trace in Node
    Error.captureStackTrace?.(this, MyError);
  }
}

/**
 * ===============================================
 * helper auto-map your error catalog to responses
 * ===============================================
 */

type ErrorRef = {
  code: keyof typeof errors
  subKey: string | string[]   // single or multiple sub-keys
}

export function errorResponses(refs: ErrorRef[]) {
  const res: Record<number, z.ZodObject<z.ZodRawShape>> = {}

  refs.forEach(({ code, subKey }) => {
    const entry = errors[code]
    if (!entry) return

    const httpCode = entry.httpCode ?? 500

    const subKeys = Array.isArray(subKey) ? subKey : [subKey]

    const messageValues: string[] = []
    const errorValues: string[] = []

    subKeys.forEach(sk => {
      const sub = (entry as Record<string, unknown>)[sk]
      if (
        sub &&
        typeof sub === 'object' &&
        sub !== null &&
        'message' in sub &&
        'error' in sub &&
        typeof sub.message === 'string' &&
        typeof sub.error === 'string'
      ) {
        messageValues.push(sub.message)
        errorValues.push(sub.error)
      }
    })

    if (messageValues.length === 0) return

    const schema = z.object({
      success: z.literal(false),
      httpCode: z.literal(httpCode),
      code: z.literal(code),
      message: messageValues.length === 1
        ? z.literal(messageValues[0])
        : z.enum(messageValues as [string, ...string[]]),
      error: errorValues.length === 1
        ? z.literal(errorValues[0])
        : z.enum(errorValues as [string, ...string[]])
    }).describe(code.replaceAll('_', ' '))
      .meta({
        examples: Object.fromEntries(
          subKeys.map((sk, i) => [
            sk.replaceAll('_', ' '),
            {
              value: {
                success: false,
                httpCode: httpCode,
                code: code,
                message: messageValues[i],
                error: errorValues[i],
              }
            }
          ])
        )
      });

    res[httpCode] = schema
  })

  return res
}

// ===============================================
// api responses helper fun
// ===============================================

type SubEntry = { message: string; error: string }

type ErrorCatalogEntry = {
  httpCode: number
  MESSAGE?: string
  DETAIL?: string
} & Record<string, SubEntry | number | string | undefined>


type DetailResponseEntry = OpenAPIV3.ResponseObject


type ApiResponsesInput = {
  success: Record<number, z.ZodTypeAny>
  error: ErrorRef[]
}

type ZodWithMeta = z.ZodTypeAny & {
  def: {
    metadata?: {
      examples?: object[]
    }
  }
}

export function apiResponses({ success, error }: ApiResponsesInput): Record<number, DetailResponseEntry> {
  const detailResponses: Record<number, DetailResponseEntry> = {}

  // success
  Object.entries(success).forEach(([code, schema]) => {
    const metaSchema = schema as ZodWithMeta
    const zodExamples = metaSchema.def.metadata?.examples

    const examples = zodExamples
      ? Object.fromEntries(
        zodExamples.map((ex, i) => [
          `example-${i + 1}`,
          { value: ex }
        ])
      )
      : undefined

    detailResponses[Number(code)] = {
      description: `Response for status ${code}`,
      content: {
        'application/json': {
          schema: { $ref: '#/components/schemas/SuccessResponse' },
          examples,
        }
      }
    }
  })

  // errors
  error.forEach(({ code, subKey }) => {
    const entry = errors[code] as ErrorCatalogEntry
    if (!entry) return

    const httpCode = entry.httpCode ?? 500
    const subKeys = Array.isArray(subKey) ? subKey : [subKey]

    const messageValues: string[] = []
    const errorValues: string[] = []

    subKeys.forEach(sk => {
      const sub = entry[sk]
      if (
        sub !== null &&
        typeof sub === 'object' &&
        'message' in sub &&
        'error' in sub &&
        typeof sub.message === 'string' &&
        typeof sub.error === 'string'
      ) {
        messageValues.push(sub.message)
        errorValues.push(sub.error)
      }
    })

    if (messageValues.length === 0) return

    const examples = Object.fromEntries(
      subKeys.map((sk, i) => [
        sk.replaceAll('_', ' '),
        {
          value: {
            success: false,
            httpCode,
            code,
            message: messageValues[i],
            error: errorValues[i],
          }
        }
      ])
    )

    detailResponses[httpCode] = {
      description: typeof entry.DETAIL === 'string' ? entry.DETAIL : code.replaceAll('_', ' '),
      content: {
        'application/json': {
          schema: { $ref: '#/components/schemas/ErrorResponse' },
          examples,
        }
      }
    }
  })

  return detailResponses
}