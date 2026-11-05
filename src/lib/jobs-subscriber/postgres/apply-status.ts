/**
 * Persist pipeline status via fn_pipeline_apply_status.
 * eventId is the DB idempotency key (must match Kafka payload; never regenerated here).
 */

import { getOnPremPool } from "@/lib/db-onprem";
import { MyError, errors } from "@/lib/errors";
import type {
  PipelineStage,
  PipelineStatusValue,
} from "@/schemas/pipeline-status.schema";
import type { ApplyStatusResult } from "@/types/global.type";

export async function applyStatus(args: {
  eventId: string;
  correlationId: string;
  stage: PipelineStage;
  status: PipelineStatusValue;
  timestamp: string;
  details?: Record<string, unknown>;
}): Promise<ApplyStatusResult> {
  const pool = await getOnPremPool({ poolKey: "service" });
  const client = await pool.connect();
  let inTxn = false;

  try {
    await client.query("BEGIN");
    inTxn = true;

    const res = await client.query(
      `SELECT result, replayed, assigned_sequence, current_stage, current_status
         FROM fn_pipeline_apply_status(
           $1::uuid,
           $2::uuid,
           $3::varchar,
           $4::varchar,
           $5::timestamptz,
           $6::jsonb
         )`,
      [
        args.eventId,
        args.correlationId,
        args.stage,
        args.status,
        args.timestamp,
        args.details ? JSON.stringify(args.details) : null,
      ],
    );

    await client.query("COMMIT");
    inTxn = false;

    const row = res.rows[0] as
      | {
          result: string;
          replayed: boolean;
          assigned_sequence: number;
          current_stage: string;
          current_status: string;
        }
      | undefined;

    if (!row || row.result !== "SUCCESS") {
      throw new MyError({
        code: "INTERNAL_SERVER_ERROR",
        message: errors.INTERNAL_SERVER_ERROR.DATABASE_ERROR.message,
        error: "fn_pipeline_apply_status did not return SUCCESS",
      });
    }

    return {
      result: row.result,
      replayed: Boolean(row.replayed),
      assignedSequence: Number(row.assigned_sequence),
      currentStage: row.current_stage,
      currentStatus: row.current_status,
    };
  } catch (err: unknown) {
    if (inTxn) {
      await client.query("ROLLBACK").catch(() => {});
    }
    if (err instanceof MyError) throw err;
    throw new MyError({
      code: "INTERNAL_SERVER_ERROR",
      message: errors.INTERNAL_SERVER_ERROR.DATABASE_ERROR.message,
      error: err instanceof Error ? err.message : String(err),
    });
  } finally {
    client.release();
  }
}
