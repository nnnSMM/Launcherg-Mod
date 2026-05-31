import { PlayStatus, type PlayStatus as PlayStatusType } from "@/lib/types";
import type { AttributeKey } from "@/components/Sidebar/searchAttributes";

export const playStatusLabel: Record<PlayStatusType, string> = {
  [PlayStatus.Unplayed]: "未プレイ",
  [PlayStatus.Playing]: "プレイ中",
  [PlayStatus.Cleared]: "クリア済み",
  [PlayStatus.Interrupted]: "中断",
  [PlayStatus.LegacyShelved]: "中断",
};

export const playStatusShortLabel: Record<PlayStatusType, string> = {
  [PlayStatus.Unplayed]: "未",
  [PlayStatus.Playing]: "PLAY",
  [PlayStatus.Cleared]: "CLEAR",
  [PlayStatus.Interrupted]: "STOP",
  [PlayStatus.LegacyShelved]: "STOP",
};

export const playStatusFilterKey: Record<PlayStatusType, AttributeKey> = {
  [PlayStatus.Unplayed]: "unplayed",
  [PlayStatus.Playing]: "playing",
  [PlayStatus.Cleared]: "cleared",
  [PlayStatus.Interrupted]: "interrupted",
  [PlayStatus.LegacyShelved]: "interrupted",
};

export const playStatusIcon: Record<PlayStatusType, string> = {
  [PlayStatus.Unplayed]: "i-material-symbols-play-circle-outline-rounded",
  [PlayStatus.Playing]: "i-material-symbols-pause-circle-outline-rounded",
  [PlayStatus.Cleared]: "i-material-symbols-check-circle-outline-rounded",
  [PlayStatus.Interrupted]: "i-material-symbols-stop-circle-outline-rounded",
  [PlayStatus.LegacyShelved]: "i-material-symbols-stop-circle-outline-rounded",
};

