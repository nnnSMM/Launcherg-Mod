export const stat = async (path: string) => {
  console.log("[Mock Tauri FS] stat:", path);
  return { isFile: true, isDirectory: false, size: 0, modifiedAt: new Date() };
};

export const readFile = async (path: string) => {
  console.log("[Mock Tauri FS] readFile:", path);
  return new Uint8Array();
};
