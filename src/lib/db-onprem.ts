import { runtimeTag } from "@/lib/constants";
import { env } from "@/lib/env";
import { Pool, type PoolConfig } from "pg";

// ===============================================
// 1. Config Registry
// ===============================================

const baseConfig: PoolConfig = {
    host: env.ONPREM_DB_SERVER,
    port: env.ONPREM_DB_PORT,
    database: env.ONPREM_DB_DATABASE,
    application_name: `${env.APP_NAME}@${runtimeTag}`,
    max: 5,
    idleTimeoutMillis: 120000,
};

const predefinedConfigs = {
    default: {
        ...baseConfig,
        user: env.ONPREM_DB_USER_SERVICE,
        password: env.ONPREM_DB_PASSWORD_SERVICE,
    },
    service: {
        ...baseConfig,
        user: env.ONPREM_DB_USER_SERVICE,
        password: env.ONPREM_DB_PASSWORD_SERVICE,
    },
};

// ===============================================
// 2. Types
// ===============================================

type PredefinedKey = keyof typeof predefinedConfigs;
export type PoolKey = PredefinedKey | (string & {});

const pools = new Map<string, Pool>();
const connecting = new Map<string, Promise<Pool>>();

// ===============================================
// 3. Public API
// ===============================================

export async function getOnPremPool(props?: {
    poolKey?: PredefinedKey;
}): Promise<Pool>;

export async function getOnPremPool(props: {
    config: PoolConfig;
    poolKey: string;
}): Promise<Pool>;

export async function getOnPremPool(
    props: {
        config?: PoolConfig;
        poolKey?: string;
    } = {}
): Promise<Pool> {
    const { config, poolKey = "default" } = props;

    const usedConfig = config ?? predefinedConfigs[poolKey as PredefinedKey];

    if (!usedConfig) {
        throw new Error(`No configuration found for pool key "${poolKey}"`);
    }

    const existing = pools.get(poolKey);

    if (existing) {
        return existing;
    }

    const pending = connecting.get(poolKey);

    if (pending) {
        return pending;
    }

    const promise = (async () => {
        try {
            const pool = new Pool(usedConfig);

            await pool.query("SELECT 1");

            pools.set(poolKey, pool);

            console.log(`_/ On-prem PostgreSQL connected (${poolKey})`);

            return pool;
        } catch (err) {
            console.error(
                `X On-prem PostgreSQL connection failed (${poolKey}):`,
                err
            );
            throw err;
        } finally {
            connecting.delete(poolKey);
        }
    })();

    connecting.set(poolKey, promise);

    return promise;
}

// ===============================================
// Helpers
// ===============================================

export function getActivePool(): Pool | null {
    return pools.get("default") ?? null;
}

export function isDbConnecting(): boolean {
    return connecting.has("default") && !pools.has("default");
}

export async function closeAllPools() {
    await Promise.all(
        [...pools.values()].map((pool) => pool.end().catch(() => {}))
    );

    pools.clear();
    connecting.clear();
}