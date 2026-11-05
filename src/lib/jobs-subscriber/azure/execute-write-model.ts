/**
 * Azure write-model SP execution for Jobs.
 * Phase 1 contract: 02spinsertRawAppliedJobs(@correlationId, @input)
 * Notes shape: { rowsInserted, message?, replayed? } — no fromId/toId.
 */
import { getAzureSQLDbPool } from "@/lib/db";
import { MyError, errors } from "@/lib/errors";
import { writeModelSpNotesSchema } from "@/schemas/global.schema";
import type { JobsRecord } from "@/schemas/jobs-event.schema";
import type { WriteModelSpResult } from "@/types/global.type";
import sql from "mssql";

export async function executeWriteModel({
  procedureName,
  correlationId,
  records,
}: {
  procedureName: string;
  correlationId: string;
  records: JobsRecord[];
}): Promise<WriteModelSpResult> {
  try {
    if (!/^[a-zA-Z0-9_."\[\]]+$/.test(procedureName) || procedureName.length > 200) {
      throw new MyError({
        code: "BAD_REQUEST",
        message: errors.BAD_REQUEST.INVALID_INPUT.message,
        error: `Invalid write-model procedure name: ${procedureName}`,
      });
    }

    const pool = await getAzureSQLDbPool({ poolKey: "service" });

    // Azure SP maps $.id → emailId; keep object-array contract
    const inputJson = JSON.stringify(records);

    const result = await pool
      .request()
      .input("correlationId", sql.UniqueIdentifier, correlationId)
      .input("input", sql.NVarChar(sql.MAX), inputJson)
      .execute(procedureName);

    const row = result.recordset?.[0] as
      | { result?: string; notes?: string }
      | undefined;

    if (!row || row.result !== "SUCCESS" || !row.notes) {
      throw new MyError({
        code: "INTERNAL_SERVER_ERROR",
        message: errors.INTERNAL_SERVER_ERROR.DATABASE_ERROR.message,
        error: "Write-model SP did not return SUCCESS with notes",
      });
    }

    const notes = writeModelSpNotesSchema.parse(JSON.parse(row.notes));

    return {
      success: true,
      data: {
        rowsInserted: notes.rowsInserted,
        message: notes.message,
        replayed: Boolean(notes.replayed),
      },
    };
  } catch (err: unknown) {
    if (err instanceof MyError) throw err;

    throw new MyError({
      code: "INTERNAL_SERVER_ERROR",
      message: errors.INTERNAL_SERVER_ERROR.DATABASE_ERROR.message,
      error: err instanceof Error ? err.message : "Azure write-model failure",
    });
  }
}