export type PingMessage = { type: "ping" };
export type MemoMessage = {
  type: "memo";
  text: string;
  gameId: number;
};
export type InitMessage = { type: "init"; gameId: number };
export type InitResponseMessage = {
  type: "init_response";
  gameId: number;
  initialMemo: MemoMessage;
};
export type TakeScreenshotMessage = {
  type: "take_screenshot";
  gameId: number;
  cursorLine: number;
};
export type ImageMetadataMessage = {
  type: "image_metadata";
  path: string;
  key: number;
  totalChunkLength: number;
  mimeType: string;
};

export type LocalMessage =
  | PingMessage
  | MemoMessage
  | InitResponseMessage
  | ImageMetadataMessage;
export type RemoteMessage =
  | PingMessage
  | MemoMessage
  | InitMessage
  | TakeScreenshotMessage;

const isObject = (value: unknown): value is Record<string, unknown> =>
  !!value && typeof value === "object";

const isFiniteNumber = (value: unknown): value is number =>
  typeof value === "number" && Number.isFinite(value);

export const isRemoteMessage = (value: unknown): value is RemoteMessage => {
  if (!isObject(value) || typeof value.type !== "string") {
    return false;
  }

  switch (value.type) {
    case "ping":
      return true;
    case "memo":
      return typeof value.text === "string" && isFiniteNumber(value.gameId);
    case "init":
      return isFiniteNumber(value.gameId);
    case "take_screenshot":
      return (
        isFiniteNumber(value.gameId) && Number.isInteger(value.cursorLine)
      );
    default:
      return false;
  }
};

export const parseRemoteMessage = (data: string): RemoteMessage | null => {
  try {
    const parsed = JSON.parse(data) as unknown;
    if (isRemoteMessage(parsed)) {
      return parsed;
    }
  } catch (error) {
    console.warn("[skyWay] Ignoring invalid remote message JSON.", error);
    return null;
  }

  console.warn("[skyWay] Ignoring unexpected remote message payload.", data);
  return null;
};
