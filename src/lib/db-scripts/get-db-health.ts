import { getAzureSQLDbPool, isDbConnecting } from "@/lib/db";

export async function getDbHealth(): Promise<
  "connected" | "connecting" | "disconnected"
> {
  try {
    if (isDbConnecting()) {
      return "connecting";
    }

    const pool = await getAzureSQLDbPool({ poolKey: "default" });

    await pool.request().query("SELECT 1");
    return "connected";
  } catch {
    return "disconnected";
  }
}
