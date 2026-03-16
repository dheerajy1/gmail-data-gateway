import { getAzureSQLDbPool } from "@/lib/db";
import { RequestError } from "mssql";

type SpOutput = {
  result: "SUCCESS";
  date: string;
};

type SpGetLastDateEmailsError = {
  message: string;
  number?: number;
};

export async function spGetLastDateEmails(): Promise<{
  data?: string;
  error?: SpGetLastDateEmailsError;
}> {
  const pool = await getAzureSQLDbPool({ poolKey: "service" });

  try {
    const result = await pool
      .request()
      .execute("[02sjobsData].[03spgetLastDateAppliedJobs]");

    const row = result.recordset?.[0] as SpOutput;

    if (!row || row.result !== "SUCCESS") {
      return {
        error: {
          message: "Stored procedure did not return SUCCESS"
        }
      };
    }

    return {
      data: row.date,
    };

  } catch (err: unknown) {

    // FIXME: test
    // console.error(`src/lib/db-scripts/sp-get-last-date-emails.ts error`, err);

    if (err instanceof RequestError) {
      return {
        error: {
          message: err.message,
          number: err.number
        }
      };
    }

    return {
      error: {
        message:
          err instanceof Error
            ? err.message
            : "Unknown database error"
      }
    };
  }
}