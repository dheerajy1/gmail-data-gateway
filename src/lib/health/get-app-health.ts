import { AppHealthDataSchemaType } from "@/schemas/health.schema";
import packageJson from "@@/package.json";

export function getAppHealth(): AppHealthDataSchemaType {
  const mem = process.memoryUsage();

  const heapTotalMb = Math.round(mem.heapTotal / (1024 * 1024));
  const heapUsedMb = Math.round(mem.heapUsed / (1024 * 1024));

  return {
    status: "UP",
    version: packageJson.version ?? "0.0.0",
    uptimeMs: Math.round(process.uptime() * 1000),
    timestampUtc: new Date().toISOString(),
    systemMemoryMetrics: {
      rssMb: Math.round(mem.rss / (1024 * 1024)),
      heapTotalMb,
      heapUsedMb,
      externalMb: Math.round(mem.external / (1024 * 1024)),
      arrayBuffersMb: Math.round((mem.arrayBuffers ?? 0) / (1024 * 1024)),
      heapUsagePercent:
        heapTotalMb === 0 ? 0 : Math.round((heapUsedMb / heapTotalMb) * 100),
    },
  };
}
