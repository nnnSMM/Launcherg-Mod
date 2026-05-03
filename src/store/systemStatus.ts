import { writable } from "svelte/store";

type SystemStatus = {
  isInitializing: boolean;
  title: string;
  message: string;
  detail: string;
};

export const systemStatus = writable<SystemStatus>({
  isInitializing: false,
  title: "",
  message: "",
  detail: "",
});
