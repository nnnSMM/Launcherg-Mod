import { describe, expect, it } from "vitest";
import {
  getOrCreateMobileCompanionRoomId,
  MOBILE_COMPANION_ROOM_ID_KEY,
} from "./mobileCompanionRoom";

const createMemoryStorage = (initialValue?: string) => {
  const values = new Map<string, string>();
  if (initialValue !== undefined) {
    values.set(MOBILE_COMPANION_ROOM_ID_KEY, initialValue);
  }

  return {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => {
      values.set(key, value);
    },
  };
};

describe("getOrCreateMobileCompanionRoomId", () => {
  it("reuses an existing companion room id", () => {
    const storage = createMemoryStorage("room-existing");

    expect(
      getOrCreateMobileCompanionRoomId(() => "room-new", storage),
    ).toBe("room-existing");
  });

  it("creates and stores a room id when none exists", () => {
    const storage = createMemoryStorage();

    expect(
      getOrCreateMobileCompanionRoomId(() => "room-new", storage),
    ).toBe("room-new");
    expect(storage.getItem(MOBILE_COMPANION_ROOM_ID_KEY)).toBe("room-new");
  });

  it("falls back to a fresh id when storage is unavailable", () => {
    const storage = {
      getItem: () => {
        throw new Error("storage denied");
      },
      setItem: () => undefined,
    };

    expect(
      getOrCreateMobileCompanionRoomId(() => "room-fallback", storage),
    ).toBe("room-fallback");
  });
});
