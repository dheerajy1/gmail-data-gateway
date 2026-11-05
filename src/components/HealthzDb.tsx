import { Layout } from "@/components/Layout";
import { AlertTriangle, CheckCircle, XCircle } from "lucide-react";
import React from "react";
import { DbHealthDataSchemaType } from "@/schemas/health.schema";

type Props = {
  title: string;
  response: {
    success: boolean;
    httpCode: number;
    message: string;
    data: DbHealthDataSchemaType;
  };
};

export function HealthzDb({ title, response }: Props): React.ReactElement {
  const { data } = response;

  return (
    <Layout title={title}>
      <div className="max-w-(--width-health) mx-auto mt-8">
        <h1 className="text-2xl font-bold mb-5 text-text-primary">
          Database Health
        </h1>
        <div className="bg-surface border border-border-thick p-4 md:p-5 rounded-lg">
          <Row label="Status">
            <StatusIcon status={data.status} />
          </Row>
          <Row label="Database">
            <DbIcon database={data.database} />
          </Row>
          <Row label="Latency (ms)">
            {data.latencyMs != null ? data.latencyMs : "—"}
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
      <span className="font-mono text-sm flex items-center gap-2">
        {children}
      </span>
    </div>
  );
}

function StatusIcon({ status }: { status: "UP" | "DOWN" }) {
  if (status === "UP") {
    return (
      <>
        <CheckCircle size={16} className="text-success" />
        UP
      </>
    );
  }
  return (
    <>
      <XCircle size={16} className="text-error" />
      DOWN
    </>
  );
}

function DbIcon({ database }: { database: DbHealthDataSchemaType["database"] }) {
  if (database === "CONNECTED") {
    return (
      <>
        <CheckCircle size={16} className="text-success" />
        CONNECTED
      </>
    );
  }
  if (database === "CONNECTING") {
    return (
      <>
        <AlertTriangle size={16} className="text-warning" />
        CONNECTING
      </>
    );
  }
  return (
    <>
      <XCircle size={16} className="text-error" />
      DISCONNECTED
    </>
  );
}
