import { defineConfig } from "@unocss/vite";
import presetWind from "@unocss/preset-wind";
import presetAttributify from "@unocss/preset-attributify";
import presetWebFonts from "@unocss/preset-web-fonts";
import presetIcons from "@unocss/preset-icons";
import transformerVariantGroup from "@unocss/transformer-variant-group";
import extractorSvelte from "@unocss/extractor-svelte";

export default defineConfig({
  presets: [
    presetAttributify(),
    presetWind(),
    presetIcons(),
    extractorSvelte(),
    presetWebFonts({
      fonts: {
        sans: [
          {
            name: "Noto Sans JP",
            weights: ["400", "500", "700"],
          },
        ],
        logo: [
          {
            name: "Space Mono",
            weights: ["400"],
          },
        ],
      },
    }),
  ],
  transformers: [transformerVariantGroup()],
  shortcuts: {
    glass: "bg-bg-primary/40 backdrop-blur-xl border-t-1 border-white/10 shadow-lg",
    "glass-card": "bg-bg-secondary/40 backdrop-blur-md border border-white/5 hover:bg-bg-secondary/60 transition-all duration-300 shadow-md hover:shadow-xl hover:-translate-y-1",
    "glass-hover": "hover:bg-white/10 transition-colors duration-200",
    "focus-ring": "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-accent focus-visible:ring-offset-2 focus-visible:ring-offset-bg-primary",
    "btn-primary": "bg-gradient-to-r from-accent-accent to-accent-accent/80 text-white shadow-lg shadow-accent-accent/20 hover:shadow-accent-accent/40 hover:scale-105 active:scale-95 transition-all duration-200",
  },
  theme: {
    colors: {
      accent: {
        accent: "#487AF9",
        success: "#347d39",
        edit: "#116329",
        warning: "#c69026",
        error: "#EA4E60",
      },
      bg: {
        primary: "#0f1115", // Darker, deeper background
        secondary: "#1a1d24",
        tertiary: "#262a33",
        disabled: "#181818",
        button: "rgba(255, 255, 255, 0.1)",
        buttonHover: "rgba(255, 255, 255, 0.15)",
        backdrop: "#0d1117",
        successDisabled: "rgba(35,134,54,0.6)",
        successHover: "#46954a",
        warning: "#37332a",
        t_primary: "#adbac7",
        t_secondary: "#CDD9E5",
        t_tertiary: "#768390",
      },
      ui: { tertiary: "#636e7b" },
      border: {
        primary: "rgba(255, 255, 255, 0.08)",
        button: "rgba(255, 255, 255, 0.1)",
        buttonHover: "rgba(255, 255, 255, 0.2)",
        warning: "#AE7C14",
        successDisabled: "rgba(35,134,54,0.6)",
      },
      text: {
        primary: "#f0f6fc",
        secondary: "#c9d1d9",
        tertiary: "#8b949e",
        link: "#58a6ff",
        white: "#FFFFFF",
        disabled: "#484f58",
        successDisabled: "rgba(255,255,255,0.5)",
        b_primary: "#22272e",
        b_secondary: "#2d333b",
        b_tertiary: "#323942",
      },
    },
    fontSize: {
      body: ["1rem", "160%"],
      body2: [".875rem", "160%"],
      body3: [".8rem", "160%"],
      h1: ["2rem", "130%"], // Larger H1
      h2: ["1.5rem", "140%"],
      h3: ["1.25rem", "145%"],
      h4: ["1.125rem", "145%"],
      caption: [".75rem", "142%"],
      input: [".875rem", "100%"],
    },
    boxShadow: {
      'glass': '0 8px 32px 0 rgba(0, 0, 0, 0.37)',
    }
  },
});
