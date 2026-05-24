export const relaunch = async () => {
  console.log("[Mock Tauri Process] relaunch");
};

export const exit = async (code = 0) => {
  console.log("[Mock Tauri Process] exit:", code);
};

