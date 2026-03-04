import { envSchema } from "@/types/env";

export const env = envSchema.parse(process.env);
