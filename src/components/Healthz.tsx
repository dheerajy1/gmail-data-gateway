import { Layout } from "@/components/Layout";
import { AlertTriangle, CheckCircle, XCircle } from "lucide-react";
import React from "react";

type Props = {
  title: string;
  response: {
    success: boolean;
    httpCode: number;
    message: string;
    data: { app: string; db: string; timestamp: string; };
  };
};

export function Healthz({ title, response }: Props): React.ReactElement {
  return (
    <Layout title={title}>
      <div className="max-w-(--width-health) mx-auto mt-8">
        <h1 className="text-2xl font-bold mb-5 text-text-primary">Health Status</h1>
        <div className="bg-surface border border-border-thick p-4 md:p-5 rounded-lg">
          <div className="flex justify-between py-2 border-b border-border">
            <span className="font-semibold text-text-muted">App</span>
            <span className="font-mono">{response.data.app}</span>
          </div>

          <div className="flex justify-between py-2 border-b border-border">
            <span className="font-semibold text-text-muted">Database</span>
            <span className="flex items-center gap-2 font-mono">
              {response.data.db === "connected" && (
                <>
                  <CheckCircle size={16} className="text-success" />
                  <span>connected</span>
                </>
              )}
              {response.data.db === "connecting" && (
                <>
                  <AlertTriangle size={16} className="text-warning" />
                  <span>connecting</span>
                </>
              )}
              {response.data.db !== "connected" && response.data.db !== "connecting" && (
                <>
                  <XCircle size={16} className="text-error" />
                  <span>{response.data.db}</span>
                </>
              )}
            </span>
          </div>

          <div className="flex justify-between py-2">
            <span className="font-semibold text-text-muted">Timestamp</span>
            <span className="font-mono text-sm">{response.data.timestamp}</span>
          </div>
        </div>
      </div>
    </Layout>
  );
}
