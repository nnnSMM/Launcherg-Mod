import { PlayStatus, type PlayStatus as PlayStatusType } from "@/lib/types";
import type { AttributeKey } from "@/components/Sidebar/searchAttributes";

export const playStatusLabel: Record<PlayStatusType, string> = {
  [PlayStatus.Unplayed]: "未プレイ",
  [PlayStatus.Playing]: "プレイ中",
  [PlayStatus.Cleared]: "クリア済み",
  [PlayStatus.Multiple]: "複数進行",
  [PlayStatus.Shelved]: "棚上げ",
};

export const playStatusShortLabel: Record<PlayStatusType, string> = {
  [PlayStatus.Unplayed]: "未",
  [PlayStatus.Playing]: "PLAY",
  [PlayStatus.Cleared]: "CLEAR",
  [PlayStatus.Multiple]: "MULTI",
  [PlayStatus.Shelved]: "HOLD",
};

export const playStatusFilterKey: Record<PlayStatusType, AttributeKey> = {
  [PlayStatus.Unplayed]: "unplayed",
  [PlayStatus.Playing]: "playing",
  [PlayStatus.Cleared]: "cleared",
  [PlayStatus.Multiple]: "multiple",
  [PlayStatus.Shelved]: "shelved",
};

export const playStatusIcon: Record<PlayStatusType, string> = {
  [PlayStatus.Unplayed]: "i-material-symbols-play-circle-outline-rounded",
  [PlayStatus.Playing]: "i-material-symbols-pause-circle-outline-rounded",
  [PlayStatus.Cleared]: "i-material-symbols-check-circle-outline-rounded",
  [PlayStatus.Multiple]: "i-material-symbols:alt-route-rounded",
  [PlayStatus.Shelved]: "i-material-symbols:schedule-rounded",
};

