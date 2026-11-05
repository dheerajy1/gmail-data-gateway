import { getAzureSQLDbPool, isDbConnecting } from "@/lib/db";
import { DbHealthDataSchemaType } from "@/schemas/health.schema";

export async function getDbHealth(): Promise<DbHealthDataSchemaType> {
  if (isDbConnecting()) {
    return {
      status: "DOWN",
      database: "CONNECTING",
      latencyMs: null,
    };
  }

  const start = Date.now();

  try {
    const pool = await getAzureSQLDbPool({ poolKey: "default" });
    await pool.request().query("SELECT 1");

    return {
      status: "UP",
      database: "CONNECTED",
      latencyMs: Date.now() - start,
    };
  } catch {
    return {
      status: "DOWN",
      database: "DISCONNECTED",
      latencyMs: null,
    };
  }
}
