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
    glass: "bg-bg-primary/60 backdrop-blur-md border-white/10",
    "glass-hover": "hover:bg-bg-primary/70 transition-colors duration-200",
    "focus-ring": "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-accent focus-visible:ring-offset-2 focus-visible:ring-offset-bg-primary",
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
        primary: "#161b22", // Darker base for glass
        secondary: "#21262d",
        tertiary: "#30363d",
        disabled: "#181818",
        button: "#373e47",
        buttonHover: "#444c56",
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
        primary: "rgba(205, 217, 229, 0.1)", // More subtle border
        button: "#CDD9E5",
        buttonHover: "#768390",
        warning: "#AE7C14",
        successDisabled: "rgba(35,134,54,0.6)",
      },
      text: {
        primary: "#e6edf3", // Brighter text for better contrast
        secondary: "#d0d7de",
        tertiary: "#8b949e",
        link: "#4493f8",
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
      h1: ["1.75rem", "145%"],
      h2: ["1.5rem", "145%"],
      h3: ["1.25rem", "145%"],
      h4: ["1.125rem", "145%"],
      caption: [".75rem", "142%"],
      input: [".875rem", "100%"],
    },
  },
});
