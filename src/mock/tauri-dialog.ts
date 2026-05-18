import { pickDemoDirectory, pickDemoFile } from "@/mock/demoBrowserFiles";

type DialogOptions = {
  directory?: boolean;
};

export const open = async (options?: DialogOptions) => {
  console.log("[Mock Tauri Dialog] open:", options);

  if (options?.directory) {
    return await pickDemoDirectory();
  }

  const path = await pickDemoFile();
  return path ? { path } : null;
};
