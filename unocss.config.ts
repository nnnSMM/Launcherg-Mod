import { defineConfig } from "@unocss/vite";
import presetWind from "@unocss/preset-wind";
import presetAttributify from "@unocss/preset-attributify";
import presetWebFonts from "@unocss/preset-web-fonts";
import presetIcons from "@unocss/preset-icons";
import transformerVariantGroup from "@unocss/transformer-variant-group";
import extractorSvelte from "@unocss/extractor-svelte";

const colorVar = (name: string) => `rgba(var(--color-${name}), <alpha-value>)`;
const fixedColorVar = (name: string) => `var(--color-${name})`;

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
    "focus-ring": "focus:outline-none focus-visible:outline-none",
  },
  theme: {
    colors: {
      accent: {
        accent: colorVar("accent-accent"),
        primary: colorVar("accent-primary"),
        primaryHover: colorVar("accent-primary-hover"),
        success: colorVar("accent-success"),
        edit: colorVar("accent-edit"),
        warning: colorVar("accent-warning"),
        error: colorVar("accent-error"),
      },
      bg: {
        primary: colorVar("bg-primary"),
        secondary: colorVar("bg-secondary"),
        tertiary: colorVar("bg-tertiary"),
        disabled: colorVar("bg-disabled"),
        button: colorVar("bg-button"),
        buttonHover: colorVar("bg-button-hover"),
        backdrop: colorVar("bg-backdrop"),
        successDisabled: fixedColorVar("bg-success-disabled"),
        successHover: colorVar("bg-success-hover"),
        warning: colorVar("bg-warning"),
        t_primary: colorVar("bg-t-primary"),
        t_secondary: colorVar("bg-t-secondary"),
        t_tertiary: colorVar("bg-t-tertiary"),
      },
      ui: { tertiary: colorVar("ui-tertiary") },
      border: {
        primary: fixedColorVar("border-primary"),
        button: colorVar("border-button"),
        buttonHover: colorVar("border-button-hover"),
        warning: colorVar("border-warning"),
        successDisabled: fixedColorVar("border-success-disabled"),
      },
      text: {
        primary: colorVar("text-primary"),
        secondary: colorVar("text-secondary"),
        tertiary: colorVar("text-tertiary"),
        link: colorVar("text-link"),
        white: colorVar("text-white"),
        disabled: colorVar("text-disabled"),
        successDisabled: fixedColorVar("text-success-disabled"),
        b_primary: colorVar("text-b-primary"),
        b_secondary: colorVar("text-b-secondary"),
        b_tertiary: colorVar("text-b-tertiary"),
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
