import { getAzureSQLDbPool } from "@/lib/db";

const pool = await getAzureSQLDbPool();
const res = await pool.request().query("SELECT 1 AS ok");

console.log(res.recordset);
