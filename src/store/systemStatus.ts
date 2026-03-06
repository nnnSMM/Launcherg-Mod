import { writable } from "svelte/store";

type SystemStatus = {
  isInitializing: boolean;
  message: string;
};

export const systemStatus = writable<SystemStatus>({
  isInitializing: false,
  message: "",
});
