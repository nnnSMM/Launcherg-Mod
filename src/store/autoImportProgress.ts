import { writable } from "svelte/store";

export type AutoImportProgressState = {
  isRunning: boolean;
  total: number;
  processed: number;
  message: string;
};

const initialState = (): AutoImportProgressState => ({
  isRunning: false,
  total: 0,
  processed: 0,
  message: "",
});

export const autoImportProgress = writable<AutoImportProgressState>(
  initialState(),
);

export const startAutoImportProgress = () => {
  autoImportProgress.set({
    isRunning: true,
    total: 0,
    processed: 0,
    message: "フォルダを読み取り中",
  });
};

export const setAutoImportProgressTotal = (total: number) => {
  autoImportProgress.update((state) => ({
    ...state,
    total,
    processed: 0,
  }));
};

export const incrementAutoImportProgress = () => {
  autoImportProgress.update((state) => ({
    ...state,
    processed:
      state.total > 0 ? Math.min(state.processed + 1, state.total) : state.processed + 1,
  }));
};

export const setAutoImportProgressMessage = (message: string) => {
  autoImportProgress.update((state) => ({
    ...state,
    message,
  }));
};

export const finishAutoImportProgress = () => {
  autoImportProgress.set(initialState());
};
