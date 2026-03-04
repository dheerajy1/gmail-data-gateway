import { getAzureSQLDbPool } from "@/lib/db";
import sql, { RequestError } from "mssql";

type SpInsertRawDataEmailsRow = {
  Result: "SUCCESS";
  Message: string;
};

export async function spInsertRawDataEmails({
  data
}: { data: Array<Object> }) {
  const pool = await getAzureSQLDbPool({ poolKey: "service" });

  try {
    const inputJson = JSON.stringify(data);

    const result = await pool
      .request()
      .input("Input", sql.NVarChar(sql.MAX), inputJson)
      .execute("dbo.SP_InsertRawDataEmails");

    const row = result.recordset?.[0] as unknown as SpInsertRawDataEmailsRow;

    if (!row || row.Result !== "SUCCESS") {
      throw new Error("SP did not return SUCCESS");
    }

    return {
      success: true,
      data: row.Message,
    };

  } catch (error: unknown) {

    // FIXME: test
    // console.error(`src/lib/db-scripts/sp-Insert-raw-data-emails.ts error`, error);

    if (error instanceof RequestError) {
      return {
        success: false,
        error: {
          number: error.number,
          message: error.message
        }
      };
    }

    return {
      success: false,
      error: {
        number: -1,
        message: error instanceof Error ? error.message : "Internal server error"
      }
    };
  }
}
