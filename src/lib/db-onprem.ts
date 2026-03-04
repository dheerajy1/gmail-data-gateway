import { runtimeTag } from "@/lib/constants";
import { env } from "@/lib/env";
import sql from "mssql";

// ===============================================
// 1. Config Registry
//    Add new keys here, and IntelliSense updates automatically.
// ===============================================
const baseConfig: sql.config = {
    server: env.ONPREM_DB_SERVER,
    database: env.ONPREM_DB_DATABASE,
    user: env.ONPREM_DB_USER_SERVICE,
    password: env.ONPREM_DB_PASSWORD_SERVICE,
    options: {
        encrypt: false,
        trustServerCertificate: true,
        appName: `${env.APP_NAME}@${runtimeTag}`,
    },
    pool: { max: 1, min: 0, idleTimeoutMillis: 120000 }
};

const predefinedConfigs = {
    default: baseConfig,
    service: {
        ...baseConfig,
        user: env.ONPREM_DB_USER_SERVICE,
        password: env.ONPREM_DB_PASSWORD_SERVICE,
    }
};

// ===============================================
// 2. Type Magic
// ===============================================
type PredefinedKey = keyof typeof predefinedConfigs;
export type PoolKey = PredefinedKey | (string & {});

const pools = new Map<string, sql.ConnectionPool>();
const connecting = new Map<string, Promise<sql.ConnectionPool>>();

// ===============================================
// 3. Clean Overloads
// ===============================================

/** Use a predefined configuration (e.g. 'default', 'service') */
export async function getOnPremPool(props?: { poolKey?: PredefinedKey }): Promise<sql.ConnectionPool>;

/** Use a completely custom configuration (requires a unique poolKey) */
export async function getOnPremPool(props: { config: sql.config; poolKey: string }): Promise<sql.ConnectionPool>;

// Implementation
export async function getOnPremPool(props: { config?: sql.config; poolKey?: string } = {}): Promise<sql.ConnectionPool> {
    const { config, poolKey = 'default' } = props;
    const key = poolKey;

    // Logic: If config is provided, use it. Otherwise, look up the predefined key.
    // If the key doesn't exist in predefinedConfigs, this will throw (or you can handle gracefully).
    const usedConfig = config || predefinedConfigs[key as PredefinedKey];

    if (!usedConfig) {
        throw new Error(`No configuration found for pool key: "${key}"`);
    }

    // Return existing active connection
    if (pools.get(key)?.connected) {
        return pools.get(key)!;
    }

    // Return existing pending connection promise
    if (connecting.has(key)) {
        return connecting.get(key)!;
    }

    // Connect new
    const promise = (async () => {
        try {
            const pool = new sql.ConnectionPool(usedConfig);
            await pool.connect();
            pools.set(key, pool);
            console.log(`_/ On-prem SQL connected (${key})`);
            return pool;
        } catch (err) {
            console.error(`X On-prem SQL connection failed (${key}):`, err);
            throw err;
        } finally {
            connecting.delete(key);
        }
    })();

    connecting.set(key, promise);
    return promise;
}

// ===============================================
// Helpers
// ===============================================
export function getActivePool(): sql.ConnectionPool | null {
    return pools.get('default') ?? null;
}

export function isDbConnecting(): boolean {
    return connecting.has('default') && !pools.has('default');
}

export async function closeAllPools() {
    await Promise.all([...pools.values()].map(p => p.close().catch(() => { })));
    pools.clear();
    connecting.clear();
}
