/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_RPC_BASE_URL: string;
  readonly VITE_RPC_DEADLINE_MS: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
