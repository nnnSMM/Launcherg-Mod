import { writable } from "svelte/store";

type BackgroundState = {
  imageUrl: string | null;
  opacity: number;
};

export const backgroundState = writable<BackgroundState>({
  imageUrl: null,
  opacity: 0,
});
