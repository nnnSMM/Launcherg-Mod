export const getCurrentWindow = () => {
  console.log("[Mock Tauri Window] getCurrentWindow");
  return {
    label: "main",
    isMaximized: async () => false,
    onResized: async (cb: () => void) => {
      void cb;
      return () => {};
    },
    minimize: async () => console.log("minimize"),
    toggleMaximize: async () => console.log("toggleMaximize"),
    close: async () => console.log("close"),
    startDragging: async () => console.log("startDragging"),
  };
};
