import { fetch } from "@tauri-apps/plugin-http";

const VNDB_API_ROOT = "https://api.vndb.org/kana";
const MIN_REQUEST_INTERVAL_MS = 2000;
const REQUEST_TIMEOUT_MS = 2800;
const THROTTLE_COOLDOWN_MS = 60000;

let queue = Promise.resolve();
let lastStartedAt = 0;
let cooldownUntil = 0;

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export class VndbRequestError extends Error {
  constructor(
    message: string,
    public readonly status: number,
  ) {
    super(message);
  }
}

const runQueued = async <T>(task: () => Promise<T>): Promise<T> => {
  const previous = queue;
  let releaseQueue: () => void = () => undefined;
  queue = new Promise<void>((resolve) => {
    releaseQueue = resolve;
  });

  await previous;
  try {
    const now = Date.now();
    const waitMs = Math.max(
      cooldownUntil - now,
      lastStartedAt + MIN_REQUEST_INTERVAL_MS - now,
      0,
    );
    if (waitMs > 0) {
      await wait(waitMs);
    }
    lastStartedAt = Date.now();
    return await task();
  } finally {
    releaseQueue();
  }
};

export const requestVndbJson = async <T>(
  path: string,
  body: unknown,
): Promise<T> =>
  runQueued(async () => {
    const request = fetch(`${VNDB_API_ROOT}${path}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    const timeout = wait(REQUEST_TIMEOUT_MS).then(() => {
      throw new VndbRequestError("VNDB request timed out", 408);
    });
    const response = await Promise.race([request, timeout]);

    if (!response.ok) {
      if (response.status === 429) {
        cooldownUntil = Date.now() + THROTTLE_COOLDOWN_MS;
      }
      throw new VndbRequestError(
        `VNDB request failed: ${response.status}`,
        response.status,
      );
    }

    return (await response.json()) as T;
  });
