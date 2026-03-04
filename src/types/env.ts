import { z } from "zod";

export const envSchema = z.object({
    ONPREM_SERVER_URL: z
        .url()
        .trim()
        .min(1, { message: "ONPREM_SERVER_URL must be a valid string" })
        .describe("On-prem server url"),

    JWT_SECRET: z
        .string()
        .trim()
        .min(1, { message: "JWT_SECRET is required and cannot be empty" })
        .max(50, {
            message: "JWT_SECRET must be under 50 characters long",
        })
        .describe("JWT Secret used for authentication"),

    // ============================
    // Client
    // ============================
    CLIENT_ID: z
        .string()
        .trim()
        .min(1, { message: "CLIENT_ID is required and cannot be empty" })
        .max(50, {
            message: "CLIENT_ID must be under 50 character long",
        })
        .describe("Public client identifier"),

    CLIENT_SECRET: z
        .string()
        .trim()
        .min(1, { message: "CLIENT_SECRET is required and cannot be empty" })
        .max(50, {
            message: "CLIENT_SECRET must be under 50 characters long",
        })
        .describe("Client secret used for authentication"),

    // ============================
    // Azure SQL Server Database
    // ============================

    AZURE_DB_SERVER: z
        .string()
        .trim()
        .min(1, {
            message: "AZURE_DB_SERVER is required and cannot be empty",
        })
        .max(50, {
            message: "AZURE_DB_SERVER must be under 50 characters long",
        })
        .describe("Server name for azure sql server database"),

    AZURE_DB_DATABASE: z
        .string()
        .trim()
        .min(1, {
            message: "AZURE_DB_DATABASE is required and cannot be empty",
        })
        .max(50, {
            message: "AZURE_DB_DATABASE must be under 50 characters long",
        })
        .describe("Database name for azure sql server database"),

    // public login account
    AZURE_DB_USER_PUBLIC: z
        .string()
        .trim()
        .min(1, { message: "AZURE_DB_USER_PUBLIC is required and cannot be empty" })
        .max(50, {
            message: "AZURE_DB_USER_PUBLIC must be under 50 characters long",
        })
        .describe("Username for azure sql server database public login account"),

    AZURE_DB_PASSWORD_PUBLIC: z
        .string()
        .trim()
        .min(1, {
            message: "AZURE_DB_PASSWORD_PUBLIC is required and cannot be empty",
        })
        .max(50, {
            message: "AZURE_DB_PASSWORD_PUBLIC must be under 50 characters long",
        })
        .describe("Password for azure sql server database public login account"),

    // service login account
    AZURE_DB_USER_SERVICE: z
        .string()
        .trim()
        .min(1, { message: "AZURE_DB_USER_SERVICE is required and cannot be empty" })
        .max(50, {
            message: "AZURE_DB_USER_SERVICE must be under 50 characters long",
        })
        .describe("Username for azure sql server database service login account"),

    AZURE_DB_PASSWORD_SERVICE: z
        .string()
        .trim()
        .min(1, {
            message: "AZURE_DB_PASSWORD_SERVICE is required and cannot be empty",
        })
        .max(50, {
            message: "AZURE_DB_PASSWORD_SERVICE must be under 50 characters long",
        })
        .describe("Password for azure sql server database service login account"),

    // ============================
    // On-prem SQL Server
    // ============================

    ONPREM_DB_SERVER: z
        .string()
        .trim()
        .min(1, {
            message: "ONPREM_DB_SERVER is required and cannot be empty",
        })
        .max(50, {
            message: "ONPREM_DB_SERVER must be under 50 characters long",
        })
        .describe("Server name for On-prem sql server"),

    ONPREM_DB_DATABASE: z
        .string()
        .trim()
        .min(1, {
            message: "ONPREM_DB_DATABASE is required and cannot be empty",
        })
        .max(50, {
            message: "ONPREM_DB_DATABASE must be under 50 characters long",
        })
        .describe("Database name for On-prem db sql login account"),

    // service login account
    ONPREM_DB_USER_SERVICE: z
        .string()
        .trim()
        .min(1, { message: "ONPREM_DB_USER_SERVICE is required and cannot be empty" })
        .max(50, {
            message: "ONPREM_DB_USER_SERVICE must be under 50 characters long",
        })
        .describe("Username for On-prem sql server database service login account"),

    ONPREM_DB_PASSWORD_SERVICE: z
        .string()
        .trim()
        .min(1, {
            message: "ONPREM_DB_PASSWORD_SERVICE is required and cannot be empty",
        })
        .max(50, {
            message: "ONPREM_DB_PASSWORD_SERVICE must be under 50 characters long",
        })
        .describe("Password for On-prem sql server database service login account"),

    APP_NAME: z
        .string()
        .trim()
        .min(1, { message: "APP_ENV is required and cannot be empty" })
        .max(50, {
            message: "APP_ENV must be under 50 characters long",
        })
        .describe("App environment"),

    // ============================
    // Kafka
    // ============================

    KAFKA_API_KEY: z
        .string()
        .trim()
        .min(1, { message: "CLIENT_ID is required and cannot be empty" })
        .max(50, {
            message: "CLIENT_ID must be under 50 character long",
        })
        .describe("Kafka api key used for authentication"),

    KAFKA_API_KEY_SECRET: z
        .string()
        .trim()
        .min(1, { message: "CLIENT_SECRET is required and cannot be empty" })
        .max(300, {
            message: "CLIENT_SECRET must be under 50 characters long",
        })
        .describe("Kafka api secret used for authentication"),

    KAFKA_TOPIC: z
        .string()
        .trim()
        .min(1, { message: "APP_ENV is required and cannot be empty" })
        .max(50, {
            message: "APP_ENV must be under 50 characters long",
        })
        .describe("Kafka topic"),

    SQL_JOB_NAME: z
        .string()
        .trim()
        .min(1, { message: "SQL_JOB_NAME is required and cannot be empty" })
        .max(50, {
            message: "SQL_JOB_NAME must be under 50 characters long",
        })
        .describe("SQL job name"),

    // ============================
    // Google
    // ============================

    Google_CLIENT_ID: z
        .string()
        .trim()
        .min(1, { message: "Google_CLIENT_ID is required and cannot be empty" })
        .max(300, {
            message: "Google_CLIENT_ID must be under 300 character long",
        })
        .describe("Google Public client identifier"),

    Google_SECRET_ID: z
        .string()
        .trim()
        .min(1, { message: "Google_SECRET_ID is required and cannot be empty" })
        .max(50, {
            message: "Google_SECRET_ID must be under 50 characters long",
        })
        .describe("Google Client secret used for authentication"),

    Google_REFRESH_TOKEN: z
        .string()
        .trim()
        .min(1, { message: "Google_REFRESH_TOKEN is required and cannot be empty" })
        .max(300, {
            message: "Google_REFRESH_TOKEN must be under 300 character long",
        })
        .describe("Google refresh token identifier"),
});
