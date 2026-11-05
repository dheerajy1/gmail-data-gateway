import { Layout } from "@/components/Layout";
import { CheckCircle } from "lucide-react";
import React from "react";
import { AppHealthDataSchemaType } from "@/schemas/health.schema";

type Props = {
  title: string;
  response: {
    success: boolean;
    httpCode: number;
    message: string;
    data: AppHealthDataSchemaType;
  };
};

export function HealthzApp({ title, response }: Props): React.ReactElement {
  const { data } = response;

  return (
    <Layout title={title}>
      <div className="max-w-(--width-health) mx-auto mt-8">
        <h1 className="text-2xl font-bold mb-5 text-text-primary">App Health</h1>
        <div className="bg-surface border border-border-thick p-4 md:p-5 rounded-lg space-y-1">
          
          {/* Group all rows together so last:border-0 works */}
          <div>
            <Row label="Status">
              <span className="flex items-center gap-2 font-mono">
                <CheckCircle size={16} className="text-success" />
                {data.status}
              </span>
            </Row>
            <Row label="Version">{data.version}</Row>
            <Row label="Uptime (ms)">{data.uptimeMs.toLocaleString()}</Row>
            <Row label="Timestamp (UTC)">{data.timestampUtc}</Row>
          </div>

          {/* Divider after rows */}
          <div className="pt-3 mt-3 border-t border-border">
            <p className="text-sm font-semibold text-text-muted mb-2">Memory (MB)</p>
            
            {/* Group memory rows too */}
            <div>
              <Row label="RSS">{data.systemMemoryMetrics.rssMb}</Row>
              <Row label="Heap Total">{data.systemMemoryMetrics.heapTotalMb}</Row>
              <Row label="Heap Used">{data.systemMemoryMetrics.heapUsedMb}</Row>
              <Row label="External">{data.systemMemoryMetrics.externalMb}</Row>
              <Row label="Array Buffers">{data.systemMemoryMetrics.arrayBuffersMb}</Row>
              <Row label="Heap Usage %">{data.systemMemoryMetrics.heapUsagePercent}%</Row>
            </div>
          </div>
        </div>
      </div>
    </Layout>
  );
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex justify-between py-2 border-b border-border last:border-0">
      <span className="font-semibold text-text-muted">{label}</span>
      <span className="font-mono text-sm">{children}</span>
    </div>
  );
}
