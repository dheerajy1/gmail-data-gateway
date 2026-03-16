import { getAzureSQLDbPool } from "@/lib/db";
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
      throw new Error("SP did not return SUCCESS");
    }

    return {
      success: true,
      data: { rowsInserted: notes.rowsInserted },
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
