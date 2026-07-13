/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly BITNODE_RPC_BASE_URL: string;
  readonly BITNODE_RPC_DEADLINE_MS: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
