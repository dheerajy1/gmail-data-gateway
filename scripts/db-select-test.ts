import { getPool } from "../src/lib/db.js";

const pool = await getPool();
const res = await pool.request().query("SELECT 1 AS ok");

console.log(res.recordset);
