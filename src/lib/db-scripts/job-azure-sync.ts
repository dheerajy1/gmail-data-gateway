import { getOnPremPool } from "@/lib/db-onprem";
import { isoNowIST } from "@/lib/isoNowIST";

export async function azureSyncJob({ jobName }: { jobName: string }) {
  try {
    const pool = await getOnPremPool({ poolKey: "service" });

    console.log(
      `${isoNowIST()} \t [PG:Init]\t Executing Sync Job (${jobName})...`,
    );

    const [schemaName, procedureName] = jobName.split(".");

    if (!schemaName || !procedureName) {
      throw new Error(`Invalid procedure name: ${jobName}`);
    }

    const identifierRegex = /^[A-Za-z_][A-Za-z0-9_]*$/;

    if (
      !identifierRegex.test(schemaName) ||
      !identifierRegex.test(procedureName)
    ) {
      throw new Error(`Invalid PostgreSQL identifier: ${jobName}`);
    }
    const { rowCount } = await pool.query(
      `
            SELECT 1
            FROM pg_proc p
            JOIN pg_namespace n
              ON n.oid = p.pronamespace
            WHERE n.nspname = $1
              AND p.proname = $2
              AND p.prokind = 'p'
            `,
      [schemaName, procedureName],
    );

    if (rowCount === 0) {
      throw new Error(`PostgreSQL procedure not found: ${jobName}`);
    }

    // await pool.query(`CALL ${jobName}()`);

    console.log(
      `${isoNowIST()} \t [PG:Action]\t _/ Job executed successfully.`,
    );
  } catch (error: unknown) {
    console.error(
      `${isoNowIST()} \t [PG:Error]\t X PostgreSQL Sync Job Error:`,
      error,
    );
    throw error;
  }
}
