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
        primary: "rgba(34, 39, 46, 0.9)",
        secondary: "rgba(45, 51, 59, 0.9)",
        tertiary: "rgba(50, 57, 66, 0.9)",
        disabled: "#181818",
        button: "rgba(55, 62, 71, 0.9)",
        buttonHover: "rgba(68, 76, 86, 0.9)",
        backdrop: "#1C2128",
        successDisabled: "rgba(35,134,54,0.6)",
        successHover: "#46954a",
        warning: "#37332a",
        t_primary: "#adbac7",
        t_secondary: "#CDD9E5",
        t_tertiary: "#768390",
      },
      ui: { tertiary: "#636e7b" },
      border: {
        primary: "#444c56",
        button: "#CDD9E5",
        buttonHover: "#768390",
        warning: "#AE7C14",
        successDisabled: "rgba(35,134,54,0.6)",
      },
      text: {
        primary: "#adbac7",
        secondary: "#CDD9E5",
        tertiary: "#768390",
        link: "#2e7cd5",
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
