import { getAzureSQLDbPool } from "@/lib/db";
import { errors, MyError } from "@/lib/errors";
import sql, { RequestError } from "mssql";

type Notes = {
  appId: number;
};

type SpOutput = {
  result: "SUCCESS";
  notes: string;
  completedAtUTC: Date;
};

export async function upsertGoogleOAuthToken({
  id,
  refreshToken
}: {
  id: number;
  refreshToken: string | null | undefined;
}): Promise<{ data: Notes; }> {

  if (!refreshToken) {
    throw new MyError({
      code: "BAD_REQUEST",
      message: errors.BAD_REQUEST.MISSING_REFRESH_TOKEN.message,
      error: errors.BAD_REQUEST.MISSING_REFRESH_TOKEN.error,
    });
  }

  const pool = await getAzureSQLDbPool({ poolKey: "service" });

  try {
    const result = await pool
      .request()
      .input("id", sql.Int, id)
      .input("refreshToken", sql.VarChar(sql.MAX), refreshToken)
      .execute("[02sjobsData].[04spupsertGoogleOAuthToken]");

    const row = result.recordset?.[0] as SpOutput;

    const notes = JSON.parse(row.notes) as Notes;

    if (!row || row.result !== "SUCCESS") {
      throw new MyError({
        code: "INTERNAL_SERVER_ERROR",
        message: errors.INTERNAL_SERVER_ERROR.EXTRACT_GMAIL_AUTH.message,
        error: errors.INTERNAL_SERVER_ERROR.EXTRACT_GMAIL_AUTH.error,
      });
    }

    return {
      data: notes,
    };

  } catch (err: unknown) {

    // FIXME: test
    // console.error(`src/lib/db-scripts/spupsertGoogleOAuthToken.ts error`, err);

    if (err instanceof RequestError) {

      switch (err.number) {

        case 50001:
          throw new MyError({
            code: "BAD_REQUEST",
            message: errors.BAD_REQUEST.MISSING_REFRESH_TOKEN.message,
            error: errors.BAD_REQUEST.MISSING_REFRESH_TOKEN.error,
          });

        case 50002:
          throw new MyError({
            code: "NOT_FOUND",
            message: errors.NOT_FOUND.CLIENT_ID_NOT_FOUND.message,
            error: errors.NOT_FOUND.CLIENT_ID_NOT_FOUND.error,
          });

        default:
          throw new MyError({
            code: "INTERNAL_SERVER_ERROR",
            message: errors.INTERNAL_SERVER_ERROR.EXTRACT_GMAIL_AUTH.message,
            error: errors.INTERNAL_SERVER_ERROR.EXTRACT_GMAIL_AUTH.error,
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
