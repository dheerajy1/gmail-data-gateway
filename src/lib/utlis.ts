/**
 * Pause execution for specified milliseconds
 */
export const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
