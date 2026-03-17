import { getAzureSQLDbPool } from "@/lib/db";
import { errors, MyError } from "@/lib/errors";
import sql, { RequestError } from "mssql";

type Notes = {
  clientId: string;
  clientSecret: string;
  refreshToken: string;
};
type SpOutput = {
  result: "SUCCESS";
  notes: string;
  completedAtUTC: Date;
};

export async function getActiveGoogleOAuthCreds({
  id
}: {
  id: number
}): Promise<{ data: Notes }> {

  const pool = await getAzureSQLDbPool({ poolKey: "service" });

  try {
    const result = await pool
      .request()
      .input("id", sql.Int, id)
      .execute("[02sjobsData].[05spgetActiveGoogleOAuthCreds]");

    const row = result.recordset?.[0] as SpOutput;

    if (!row || row.result !== "SUCCESS") {
      throw new MyError({
        code: "INTERNAL_SERVER_ERROR",
        message: errors.INTERNAL_SERVER_ERROR.EXTRACT_GMAIL_AUTH.message,
        error: errors.INTERNAL_SERVER_ERROR.EXTRACT_GMAIL_AUTH.error,
      });
    }

    const notes = JSON.parse(row.notes) as Notes;

    return { data: notes };

  } catch (err: unknown) {

    // FIXME: test
    // console.error(`src/lib/db-scripts/spgetActiveGoogleOAuthCreds.ts error`, err);

    if (err instanceof RequestError) {

      switch (err.number) {

        case 50001:
          throw new MyError({
            code: "BAD_REQUEST",
            message: errors.BAD_REQUEST.INVALID_INPUT.message,
            error: errors.BAD_REQUEST.INVALID_INPUT.error,
          });

        case 50002:
          throw new MyError({
            code: "NOT_FOUND",
            message: errors.NOT_FOUND.CLIENT_ID_NOT_FOUND.message,
            error: errors.NOT_FOUND.CLIENT_ID_NOT_FOUND.error,
          });

        case 50003:
          throw new MyError({
            code: "UNAUTHORIZED",
            message: errors.UNAUTHORIZED.NO_ACTIVE_CREDENTIALS.message,
            error: errors.UNAUTHORIZED.NO_ACTIVE_CREDENTIALS.error,
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
