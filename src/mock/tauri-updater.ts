export const check = async () => {
  console.log("[Mock Tauri Updater] check");
  return null;
};

export type DownloadEvent =
  | { event: "Started"; data: { contentLength?: number } }
  | { event: "Progress"; data: { chunkLength: number } }
  | { event: "Finished" };

export type Update = {
  currentVersion: string;
  version: string;
  date?: string;
  body?: string;
  downloadAndInstall: (onEvent?: (event: DownloadEvent) => void) => Promise<void>;
};
