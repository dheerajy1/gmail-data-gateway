import { getAzureSQLDbPool } from "@/lib/db";
import { MyError, errors } from "@/lib/errors";
import sql, { RequestError } from "mssql";

type Notes = {
  fromId: number;
  toId: number;
  rowsInserted: number;
};

type SpOutput = {
  result: "SUCCESS";
  notes: string;
};

export async function spInsertRawDataEmails({
  data
}: { data: Array<Object> }) {
  const pool = await getAzureSQLDbPool({ poolKey: "service" });

  try {
    const inputJson = JSON.stringify(data);

    const result = await pool
      .request()
      .input("input", sql.NVarChar(sql.MAX), inputJson)
      .execute("[02sjobsData].[02spinsertRawAppliedJobs]");

    const row = result.recordset?.[0] as unknown as SpOutput;

    const notes = JSON.parse(row.notes) as Notes;

    if (!row || row.result !== "SUCCESS") {
      throw new MyError({
        code: "CONFLICT",
        message: errors.CONFLICT.DATABASE_CONFLICT.message,
        error: errors.CONFLICT.DATABASE_CONFLICT.error,
      });
    }

    return {
      data: notes.rowsInserted,
    };

  } catch (err: unknown) {

    // FIXME: test
    // console.error(`src/lib/db-scripts/sp-Insert-raw-data-emails.ts error`, err);

    if (err instanceof RequestError) {

      switch (err.number) {
        case 50001:
          throw new MyError({
            code: "BAD_REQUEST",
            message: errors.BAD_REQUEST.INVALID_JSON_INPUT.message,
            error: errors.BAD_REQUEST.INVALID_JSON_INPUT.error,
          });

        case 50002:
          throw new MyError({
            code: "BAD_REQUEST",
            message: errors.BAD_REQUEST.EMPTY_PAYLOAD.message,
            error: errors.BAD_REQUEST.EMPTY_PAYLOAD.error,
          });

        case 50003:
          throw new MyError({
            code: "BAD_REQUEST",
            message: errors.BAD_REQUEST.MALFORMED_INPUT_ROWS.message,
            error: errors.BAD_REQUEST.MALFORMED_INPUT_ROWS.error,
          });

        case 50004:
          throw new MyError({
            code: "CONFLICT",
            message: errors.CONFLICT.NO_NEW_RECORDS.message,
            error: errors.CONFLICT.NO_NEW_RECORDS.error,
          });

        default:
          throw new MyError({
            code: "CONFLICT",
            message: errors.CONFLICT.DATABASE_CONFLICT.message,
            error: errors.CONFLICT.DATABASE_CONFLICT.error,
          });
      }
    }

    throw new MyError({
      code: "INTERNAL_SERVER_ERROR",
      message: errors.INTERNAL_SERVER_ERROR.DATABASE_ERROR.message,
      error: errors.INTERNAL_SERVER_ERROR.DATABASE_ERROR.error,
    });
  }
}
