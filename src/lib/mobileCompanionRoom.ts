export const MOBILE_COMPANION_ROOM_ID_KEY =
  "launcherg-mobile-companion-room-id-v1";

type RoomStorage = Pick<Storage, "getItem" | "setItem">;

export const getOrCreateMobileCompanionRoomId = (
  createId: () => string,
  storage: RoomStorage | undefined =
    typeof localStorage !== "undefined" ? localStorage : undefined,
) => {
  if (!storage) {
    return createId();
  }

  try {
    const existingRoomId = storage
      .getItem(MOBILE_COMPANION_ROOM_ID_KEY)
      ?.trim();
    if (existingRoomId) {
      return existingRoomId;
    }

    const roomId = createId();
    storage.setItem(MOBILE_COMPANION_ROOM_ID_KEY, roomId);
    return roomId;
  } catch {
    return createId();
  }
};
