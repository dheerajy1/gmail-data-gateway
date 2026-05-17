import { getAzureSQLDbPool } from "@/lib/db";
import { errors, MyError } from "@/lib/errors";
import sql, { RequestError } from "mssql";

type App = {
    clientId: string;
    clientSecret: string;
};

export async function getGoogleOAuthApp({
    id
}: {
    id: number;
}): Promise<{ data: App }> {

    const pool = await getAzureSQLDbPool({ poolKey: "service" });

    try {
        const result = await pool
            .request()
            .input("id", sql.Int, id)
            .query(`
        SELECT clientId, clientSecret
        FROM [02sjobsData].[05tgoogleOAuthApps]
        WHERE ID = @id
      `);

        const row = result.recordset?.[0] as App;

        if (!row) {
            throw new MyError({
                code: "NOT_FOUND",
                message: errors.NOT_FOUND.CLIENT_ID_NOT_FOUND.message,
                error: errors.NOT_FOUND.CLIENT_ID_NOT_FOUND.error,
            });
        }

        return { data: row };

    } catch (err: unknown) {

        if (err instanceof RequestError) {

            switch (err.number) {

                case 50001:
                    throw new MyError({
                        code: "NOT_FOUND",
                        message: errors.NOT_FOUND.APPS_TABLE_MISSING.message,
                        error: errors.NOT_FOUND.APPS_TABLE_MISSING.error,
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
