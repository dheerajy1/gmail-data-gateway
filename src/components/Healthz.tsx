import { Layout } from "@/components/Layout";
import {
  AppHealthDataSchemaType,
  DbHealthDataSchemaType,
} from "@/schemas/health.schema";
import { AlertTriangle, CheckCircle, XCircle } from "lucide-react";
import React from "react";

type Props = {
  title: string;
  response: {
    success: boolean;
    httpCode: number;
    message: string;
    data: {
      app: AppHealthDataSchemaType;
      db: DbHealthDataSchemaType;
    };
  };
};

export function Healthz({ title, response }: Props): React.ReactElement {
  const { app, db } = response.data;

  return (
    <Layout title={title}>
      <div className="max-w-(--width-health) mx-auto mt-8 space-y-6">
        <h1 className="text-2xl font-bold text-text-primary">Health Status</h1>

        {/* App section */}
        <div className="bg-surface border border-border-thick p-4 md:p-5 rounded-lg">
          <h2 className="text-lg font-semibold mb-3 text-text-primary">
            Application
          </h2>
          <Row label="Status">
            <span className="flex items-center gap-2">
              <CheckCircle size={16} className="text-success" />
              {app.status}
            </span>
          </Row>
          <Row label="Version">{app.version}</Row>
          <Row label="Uptime (ms)">{app.uptimeMs.toLocaleString()}</Row>
          <Row label="Timestamp (UTC)">{app.timestampUtc}</Row>

          <div className="pt-3 mt-3 border-t border-border">
            <p className="text-sm font-semibold text-text-muted mb-2">
              Memory (MB)
            </p>
            <Row label="RSS">{app.systemMemoryMetrics.rssMb}</Row>
            <Row label="Heap Total">{app.systemMemoryMetrics.heapTotalMb}</Row>
            <Row label="Heap Used">{app.systemMemoryMetrics.heapUsedMb}</Row>
            <Row label="External">{app.systemMemoryMetrics.externalMb}</Row>
            <Row label="Array Buffers">
              {app.systemMemoryMetrics.arrayBuffersMb}
            </Row>
            <Row label="Heap Usage %">
              {app.systemMemoryMetrics.heapUsagePercent}%
            </Row>
          </div>
        </div>

        {/* DB section */}
        <div className="bg-surface border border-border-thick p-4 md:p-5 rounded-lg">
          <h2 className="text-lg font-semibold mb-3 text-text-primary">
            Database
          </h2>
          <Row label="Status">
            {db.status === "UP" ? (
              <span className="flex items-center gap-2">
                <CheckCircle size={16} className="text-success" /> UP
              </span>
            ) : (
              <span className="flex items-center gap-2">
                <XCircle size={16} className="text-error" /> DOWN
              </span>
            )}
          </Row>
          <Row label="Connection">
            {db.database === "CONNECTED" && (
              <span className="flex items-center gap-2">
                <CheckCircle size={16} className="text-success" /> CONNECTED
              </span>
            )}
            {db.database === "CONNECTING" && (
              <span className="flex items-center gap-2">
                <AlertTriangle size={16} className="text-warning" /> CONNECTING
              </span>
            )}
            {db.database === "DISCONNECTED" && (
              <span className="flex items-center gap-2">
                <XCircle size={16} className="text-error" /> DISCONNECTED
              </span>
            )}
          </Row>
          <Row label="Latency (ms)">
            {db.latencyMs != null ? db.latencyMs : "—"}
          </Row>
        </div>
      </div>
    </Layout>
  );
}

function Row({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex justify-between py-2 border-b border-border last:border-0">
      <span className="font-semibold text-text-muted">{label}</span>
      <span className="font-mono text-sm">{children}</span>
    </div>
  );
}
