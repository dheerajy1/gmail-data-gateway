/**
 * PostgreSQL read-model stored procedure execution.
 * Procedure name comes from the event (job.readModel).
 * Phase 1: CALL sp_kafka_apply_committed_records(uuid, jsonb, OUT...)
 */

import { getOnPremPool } from "@/lib/db-onprem";
import { MyError, errors } from "@/lib/errors";
import { readModelRowSchema } from "@/schemas/global.schema";
import type { JobsRecord } from "@/schemas/jobs-event.schema";
import type { ReadModelApplyResult } from "@/types/global.type";

export async function executeReadModel({
  procedureName,
  correlationId,
  records,
}: {
  procedureName: string;
  correlationId: string;
  records: JobsRecord[];
}): Promise<ReadModelApplyResult> {

  if (!/^[a-zA-Z0-9_."]+$/.test(procedureName) || procedureName.length > 200) {
    throw new MyError({
      code: "BAD_REQUEST",
      message: errors.BAD_REQUEST.INVALID_INPUT.message,
      error: `Invalid read-model procedure name: ${procedureName}`,
    });
  }

  const pool = await getOnPremPool({ poolKey: "service" });
  const client = await pool.connect();
  let inTxn = false;

  try {
    await client.query("BEGIN");
    inTxn = true;

    const res = await client.query(
      `CALL ${procedureName}(
         $1::uuid,
         $2::jsonb,
         NULL,
         NULL,
         NULL
       )`,
      [correlationId, JSON.stringify(records)],
    );

    await client.query("COMMIT");
    inTxn = false;

    const row = readModelRowSchema.parse(res.rows[0]);

    if (row.result !== "SUCCESS") {
      throw new MyError({
        code: "INTERNAL_SERVER_ERROR",
        message: errors.INTERNAL_SERVER_ERROR.DATABASE_ERROR.message,
        error: `${procedureName} did not return SUCCESS`,
      });
    }

    return {
      result: row.result,
      replayed: Boolean(row.replayed),
      rowsInserted: row.rows_inserted ?? null,
    };
  } catch (err: unknown) {
    if (inTxn) {
      await client.query("ROLLBACK").catch(() => {});
    }
    if (err instanceof MyError) throw err;
    throw new MyError({
      code: "INTERNAL_SERVER_ERROR",
      message: errors.INTERNAL_SERVER_ERROR.DATABASE_ERROR.message,
      error:
        err instanceof Error ? err.message : "PostgreSQL read-model failure",
    });
  } finally {
    client.release();
  }
}