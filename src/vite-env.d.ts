/// <reference types="svelte" />
/// <reference types="vite/client" />

declare const __PUBLIC_DEMO_BUILD__: boolean;

interface ImportMetaEnv {
  readonly VITE_MOBILE_COMPANION_ORIGIN?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
