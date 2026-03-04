export function isoNowIST(): string {
    const now = new Date();
    const istOffsetMs = 5.5 * 60 * 60 * 1000;
    const ist = new Date(now.getTime() + istOffsetMs);

    return ist.toISOString().replace("Z", "+05:30");
}
