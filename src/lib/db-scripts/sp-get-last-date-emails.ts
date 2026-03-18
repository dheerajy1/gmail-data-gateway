import { getAzureSQLDbPool } from "@/lib/db";
import { MyError, errors } from "@/lib/errors";
import { RequestError } from "mssql";

type SpOutput = {
  result: "SUCCESS";
  date: string;
};

export async function spGetLastDateEmails(): Promise<{ data: string }> {
  const pool = await getAzureSQLDbPool({ poolKey: "service" });

  try {
    const result = await pool
      .request()
      .execute("[02sjobsData].[03spgetLastDateAppliedJobs]");

    const row = result.recordset?.[0] as SpOutput;

    if (!row || row.result !== "SUCCESS") {
      throw new MyError({
        code: "INTERNAL_SERVER_ERROR",
        message: errors.INTERNAL_SERVER_ERROR.EXTRACT_FAILED.message,
        error: errors.INTERNAL_SERVER_ERROR.EXTRACT_FAILED.error,
      });
    }

    return {
      data: row.date,
    };

  } catch (err: unknown) {

    // FIXME: test
    // console.error(`src/lib/db-scripts/sp-get-last-date-emails.ts error`, err);

    if (err instanceof RequestError) {

      switch (err.number) {

        case 50001:
          throw new MyError({
            code: "NOT_FOUND",
            message: errors.NOT_FOUND.NO_EMAIL_DATA.message,
            error: errors.NOT_FOUND.NO_EMAIL_DATA.error,
          });

        default:
          throw new MyError({
            code: "INTERNAL_SERVER_ERROR",
            message: errors.INTERNAL_SERVER_ERROR.EXTRACT_FAILED.message,
            error: errors.INTERNAL_SERVER_ERROR.EXTRACT_FAILED.error,
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
