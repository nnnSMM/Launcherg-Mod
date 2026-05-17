export const open = async (path: string) => {
  console.log("[Mock Tauri Shell] open:", path);
  window.open(path, '_blank');
};
