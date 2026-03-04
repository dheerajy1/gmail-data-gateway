import { z } from "zod";

export const JwtPayloadSchema = z.object({
    sub: z.string(),
    role: z.enum(["admin", "user"]),
    iat: z.number(),
    exp: z.number(),
    iss: z.string().optional(),
});

export type JwtPayloadType = z.infer<typeof JwtPayloadSchema>;

// =========================
// Schema for required auth headers
// =========================
export const NoAuthHeadersSchema = z.object({
    "x-client-id": z
        .string()
        .trim()
        .min(1, { message: "x-client-id is required and cannot be empty" })
        .length("nextjs-frontend".length, "CLIENT_ID must match expected length")
        .describe("Client identifier"),
    "x-client-secret": z
        .string()
        .trim()
        .min(1, { message: "x-client-secret is required and cannot be empty" })
        .length(44, "CLIENT_SECRET must be 32 bytes base64 (openssl rand -base64 32)")
        .describe("Client secret"),
});

export const AuthHeadersSchema = z.object({
    "x-client-id": z
        .string()
        .trim()
        .min(1, { message: "x-client-id is required and cannot be empty" })
        .length("nextjs-frontend".length, "x-client-id must match expected length")
        .describe("Client identifier"),
    "x-client-secret": z
        .string()
        .trim()
        .min(1, { message: "x-client-secret is required and cannot be empty" })
        .length(44, "x-client-secret must be 32 bytes base64 (openssl rand -base64 32)")
        .describe("Client secret"),
    authorization: z
        .string()
        .regex(/^Bearer .+/, "Must be a valid Bearer token")
        .describe("JWT Bearer token"),
});

// =========================
// LOGIN REQUEST
// =========================
export const LoginRequestSchema = z.object({
    username: z.string(),
    password: z.string(),
});

export const LoginInputSchema = LoginRequestSchema
// z.object({
//     headers: withNoAuthHeadersSchema,
//     body: LoginRequestSchema,
// })

// =========================
// LOGIN RESPONSE SUCCESS 
// =========================
export const LoginResponseSchema = z.object({
    success: z.literal(true),
    httpCode: z.literal(200),
    message: z.string(),
    data: z.object({
        token: z.string(),
        expiresAt: z.string(),
    }),
}).meta({
    examples: [{
        success: true,
        httpCode: 200,
        message: "Login successful",
        data: {
            token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
            expiresAt: "2025-01-01T00:00:00.000Z"
        }
    }]
});

export type LoginResponse = z.infer<typeof LoginResponseSchema>;

// =========================
// LOGOUT REQUEST
// =========================
export const LogoutRequestSchema = z.object({
    username: z.string(),
    password: z.string(),
});

export const LogoutInputSchema = z.object({
    headers: NoAuthHeadersSchema,
    // body: LoginRequestSchema,
})
// =========================
// SIGNUP REQUEST
// =========================
export const SignupRequestSchema = z.object({
    username: z.string(),
    password: z.string(),
});

export const SignupInputSchema = z.object({
    headers: NoAuthHeadersSchema,
    body: SignupRequestSchema,
})

// =========================
// SIGNUP RESPONSE SUCCESS 
// =========================
export const SignupResponseSchema = z.object({
    success: z.literal(true),
    httpCode: z.literal(200),
    message: z.string(),
    data: z.object({
        userId: z.number(),
    }),
});

// =========================
// REFRESH TOKEN REQUEST
// =========================

export const RefreshTokenInputSchema = z.object({
    headers: z.object({
        authorization: z
            .string()
            .regex(/^Bearer .+/, "Must be a valid Bearer token")
            .describe("JWT Bearer token"),
        "x-client-id": z
            .string()
            .trim()
            .min(1, { message: "x-client-id is required and cannot be empty" })
            .length("nextjs-frontend".length, "x-client-id must match expected length")
            .describe("Client identifier"),
        "x-client-secret": z
            .string()
            .trim()
            .min(1, { message: "x-client-secret is required and cannot be empty" })
            .length(44, "x-client-secret must be 32 bytes base64 (openssl rand -base64 32)")
            .describe("Client secret"),
    }),
})

// =========================
// REFRESH TOKEN RESPONSE SUCCESS
// =========================
export const RefreshTokenResponseSchema = z.object({
    success: z.literal(true),
    httpCode: z.literal(200),
    message: z.string(),
    data: z.object({
        token: z.object({
            accessToken: z.string(),
            refreshToken: z.string(),
        }),
    }),
});

export type RefreshTokenResponse = z.infer<typeof RefreshTokenResponseSchema>;

// =========================
// ERROR RESPONSE
// =========================
export const ErrorResponseSchema = z.object({
    success: z.literal(false),
    httpCode: z.number(),
    error: z.string(),
    message: z.string(),
});

export type ErrorResponse = z.infer<typeof ErrorResponseSchema>;
