type EventPayload<T = unknown> = {
  event: string;
  payload: T;
};

type EventHandler<T = unknown> = (event: EventPayload<T>) => void;

const listeners = new Map<string, Set<EventHandler>>();

export const listen = async (
  event: string,
  handler: EventHandler,
) => {
  console.log("[Mock Tauri Event] listen:", event);
  const handlers = listeners.get(event) ?? new Set<EventHandler>();
  handlers.add(handler);
  listeners.set(event, handlers);
  return () => {
    handlers.delete(handler);
  };
};

export const emit = async (event: string, payload?: unknown) => {
  console.log("[Mock Tauri Event] emit:", event, payload);
  const handlers = listeners.get(event);
  if (!handlers) {
    return;
  }
  for (const handler of handlers) {
    handler({ event, payload });
  }
};
