import type { PlayStatus } from "@/lib/types";

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
  hideText?: boolean;
};
export type ControlStatusRequestMessage = {
  type: "control_status_request";
};
export type PauseToggleMessage = {
  type: "pause_toggle";
};
export type ScreenshotResultMessage = {
  type: "screenshot_result";
  gameId: number;
  ok: boolean;
  imagePath?: string;
  error?: string;
};
export type ControlStatusMessage = {
  type: "control_status";
  isPaused: boolean;
  error?: string;
};
export type LibraryRequestMessage = {
  type: "library_request";
};
export type RemoteGameSummary = {
  id: number;
  title: string;
  brandName: string;
  playStatus: PlayStatus;
  totalPlayTimeSeconds: number;
  lastPlayAt: string | null;
  installed: boolean;
  liked: boolean;
  thumbnailPath?: string | null;
  thumbnailWidth?: number | null;
  thumbnailHeight?: number | null;
};
export type LibraryResponseMessage = {
  type: "library_response";
  games: RemoteGameSummary[];
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
  | LibraryResponseMessage
  | ScreenshotResultMessage
  | ControlStatusMessage
  | ImageMetadataMessage;
export type RemoteMessage =
  | PingMessage
  | MemoMessage
  | InitMessage
  | LibraryRequestMessage
  | TakeScreenshotMessage
  | ControlStatusRequestMessage
  | PauseToggleMessage;

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
    case "library_request":
      return true;
    case "control_status_request":
      return true;
    case "pause_toggle":
      return true;
    case "take_screenshot":
      return (
        isFiniteNumber(value.gameId) &&
        Number.isInteger(value.cursorLine) &&
        (value.hideText === undefined || typeof value.hideText === "boolean")
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
