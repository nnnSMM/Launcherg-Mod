export type Work = {
  id: number;
  name: string;
  brandId: number;
  brandName: string;
  officialHomePage: string;
  sellday: string;
  imgUrl: string;
  statistics: Statistics;
  creators: Creators;
  musics: string[];
};

export type Statistics = {
  median: number;
  average: number;
  count: number;
  playTime: string;
};

export type Creators = {
  illustrators: Creator[];
  writers: Creator[];
  voiceActors: VoiceActor[];
};

export type Creator = {
  id: number;
  name: string;
};

export const VoiceActorImportance = {
  Main: 0,
  Sub: 1,
  Mob: 2,
} as const;

export type VoiceActor = {
  role: string;
  importance: (typeof VoiceActorImportance)[keyof typeof VoiceActorImportance];
} & Creator;

export type Collection = {
  id: number;
  name: string;
};

export const PlayStatus = { // 追加
  Unplayed: 0,
  Playing: 1,
  Cleared: 2,
} as const;
export type PlayStatus = (typeof PlayStatus)[keyof typeof PlayStatus]; // 追加

export type CollectionElement = {
  id: number; // Work.id と同じ
  gamename: string;
  gamenameRuby: string;
  brandname: string;
  brandnameRuby: string;
  sellday: string;
  isNukige: boolean;
  installAt: string | null;
  lastPlayAt: string | null;
  likeAt: string | null;
  playStatus: PlayStatus;
  totalPlayTimeSeconds: number;
  registeredAt: string;
  exePath: string;
  lnkPath: string;
  icon: string;
  thumbnail: string;
  thumbnailWidth: number | null;
  thumbnailHeight: number | null;
};

export type CollectionElementsWithLabel = {
  label: string;
  elements: CollectionElement[];
};

export type SeiyaDataPair = [string, string];

export type CollectionElementDetail = {
  collectionElementId: number;
  gamenameRuby: string;
  brandname: string;
  brandnameRuby: string;
  sellday: string;
  isNukige: boolean;
};

export type AllGameCacheOne = {
  id: number;
  gamename: string;
  thumbnailUrl: string;
};
