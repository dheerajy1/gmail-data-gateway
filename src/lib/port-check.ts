import { execSync } from "node:child_process";

export function listenIfPortFree({ port }: { port: number }): number {
  try {
    execSync(`ss -tulnp | grep -q ":${port}[[:space:]]"`, { stdio: "ignore" });

    console.error(`Error: Port ${port} already in use`);
    process.exit(1);

  } catch {
    return port;
  }
}
