import { jwtVal } from "@/lib/constants";
import { getAzureSQLDbPool } from "@/lib/db";
import { apiResponses, errors, MyError } from "@/lib/errors";
import { NoAuth } from "@/middleware/auth";
import { ErrorResponseSchema, LoginResponseSchema, NoAuthHeadersSchema } from "@/types/auth";
import { jwt } from '@elysiajs/jwt';
import { Elysia } from "elysia";
import z from "zod";

const router = new Elysia()
    .use(jwt(jwtVal))
    .use(NoAuth)
    .model({
        SuccessResponse: LoginResponseSchema,
        ErrorResponse: ErrorResponseSchema,
    })

router.post(
    "/login",
    async ({ body, jwt, status }) => {

        const { username, password } = body;

        const pool = await getAzureSQLDbPool({ poolKey: "default" });

        const result = await pool
            .request()
            .input("username", username)
            .query(`
        SELECT Username, PasswordHash, UserRole
        FROM tApiUsers02
        WHERE username = @username
      `);

        if (result.recordset.length === 0) {
            throw new MyError({
                code: "UNAUTHORIZED",
                message: errors.UNAUTHORIZED.USER_NOT_FOUND.message,
                error: errors.UNAUTHORIZED.USER_NOT_FOUND.error,
            });
        }

        const { Username, PasswordHash, UserRole } = result.recordset[0];

        const valid = await Bun.password.verify(password, PasswordHash);

        if (!valid) {
            throw new MyError({
                code: "UNAUTHORIZED",
                message: errors.UNAUTHORIZED.INVALID_CREDENTIALS.message,
                error: errors.UNAUTHORIZED.INVALID_CREDENTIALS.error,
            });
        }

        const token = await jwt.sign({
            sub: Username,
            role: UserRole,
        });

        if (!token) {
            throw new MyError({
                code: "INTERNAL_SERVER_ERROR",
                message: errors.INTERNAL_SERVER_ERROR.TOKEN_SIGNING_FAILED.message,
                error: errors.INTERNAL_SERVER_ERROR.TOKEN_SIGNING_FAILED.error,
            });
        }

        const payload = await jwt.verify(token);

        if (!payload) {
            throw new MyError({
                code: "INTERNAL_SERVER_ERROR",
                message: errors.INTERNAL_SERVER_ERROR.TOKEN_VERIFICATION_FAILED.message,
                error: errors.INTERNAL_SERVER_ERROR.TOKEN_VERIFICATION_FAILED.error,
            });
        }

        const expSeconds = payload.exp;

        if (!expSeconds) {
            throw new MyError({
                code: "INTERNAL_SERVER_ERROR",
                message: errors.INTERNAL_SERVER_ERROR.TOKEN_VERIFICATION_FAILED.message,
                error: errors.INTERNAL_SERVER_ERROR.TOKEN_VERIFICATION_FAILED.error,
            });
        }

        const expiresAt = new Date(expSeconds * 1000).toISOString();

        // ============================
        // Return response
        // ============================

        return status(200, {
            success: true,
            httpCode: 200,
            message: "Login successful",
            data: {
                token: token,
                expiresAt: expiresAt,
            },
        });
    },
    {
        noAuth: true,
        body: z.object({
            username: z.string().min(1),
            password: z.string().min(6)
        }),
        headers: NoAuthHeadersSchema,
        detail: {
            tags: ["Auth"],
            summary: "API user login",
            description: "Authenticate a user and return access token",
            responses: {
                ...apiResponses({
                    success: {
                        200: LoginResponseSchema
                    },
                    error: [
                        { code: 'UNAUTHORIZED', subKey: ['MISSING_AUTH_HEADERS', 'USER_NOT_FOUND', 'INVALID_CREDENTIALS'] },
                        { code: 'BAD_REQUEST', subKey: ['INVALID_HEADER_VALUES'] },
                        { code: 'INTERNAL_SERVER_ERROR', subKey: ['TOKEN_SIGNING_FAILED', 'TOKEN_VERIFICATION_FAILED'] }
                    ]
                })

            }
        }
    }
);

export default router;
