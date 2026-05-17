export const listen = async (
  event: string,
  handler: (event: unknown) => void,
) => {
  console.log("[Mock Tauri Event] listen:", event);
  void handler;
  return () => {}; // Unlisten function
};

export const emit = async (event: string, payload?: unknown) => {
  console.log("[Mock Tauri Event] emit:", event, payload);
};
