import { envSchema } from "@/schemas/env.schema";

let env: ReturnType<typeof envSchema.parse>;

try {
  env = envSchema.parse(process.env);
} catch (error) {
  console.error("[env] Environment validation failed");

  if (error instanceof Error) {
    console.error(error.message);
  } else {
    console.error(error);
  }

  process.exit(1);
}

export { env };