import { getOnPremPool } from "@/lib/db-onprem";
import { isoNowIST } from "@/lib/isoNowIST";
import sql from "mssql";

export async function azureSyncJob({ jobName }: { jobName: string }) {
    try {
        // Reuse your existing smart pool
        const pool = await getOnPremPool({ poolKey: "service" });

        console.log(`${isoNowIST()} \t [SQL:Init]\t Executing Agent Job...`);

        await pool.request()
            .input("targetJobName", sql.VarChar(100), jobName)
            .query(`
        EXEC msdb.dbo.sp_start_job 
        @job_name = @targetJobName;
    `);

        console.log(`${isoNowIST()} \t [SQL:Action]\t _/ Job started successfully.`);

    } catch (error: unknown) {
        // const msg = error instanceof Error ? error.message : error;
        console.error(`${isoNowIST()} \t [SQL:Error]\t X On Prem SQL Server Error:`, error);
        throw error;
    }
}
