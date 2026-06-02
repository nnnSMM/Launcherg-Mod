import { createWritable } from "@/lib/utils";

export const THEME_SETTING_KEY = "theme";

export type AppTheme = "dark" | "light";

const isAppTheme = (value: string | null | undefined): value is AppTheme =>
  value === "dark" || value === "light";

const [themeStore, getTheme] = createWritable<AppTheme>("dark");

const applyTheme = (theme: AppTheme) => {
  document.documentElement.dataset.theme = theme;
};

themeStore.subscribe(applyTheme);

export const theme = {
  subscribe: themeStore.subscribe,
  value: getTheme,
  async initialize() {
    const { commandGetAppSetting } = await import("@/lib/command");
    const saved = await commandGetAppSetting(THEME_SETTING_KEY);
    themeStore.set(isAppTheme(saved) ? saved : "dark");
  },
  async set(nextTheme: AppTheme) {
    themeStore.set(nextTheme);
    const { commandSetAppSetting } = await import("@/lib/command");
    await commandSetAppSetting(
      THEME_SETTING_KEY,
      nextTheme === "dark" ? null : nextTheme,
    );
  },
};
