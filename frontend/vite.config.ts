import { defineConfig } from "vite";
import { execSync } from "child_process";
import { readdirSync } from "fs";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import path from "path";
import babel from "@rolldown/plugin-babel";
import react, { reactCompilerPreset } from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import type { Plugin } from "vite";

function protocBuild(): Plugin {
  const protoDir = path.resolve(import.meta.dirname, "protos");

  return {
    name: "protoc-build",
    buildStart() {
      for (const file of readdirSync(protoDir)) {
        if (file.endsWith(".proto")) {
          this.addWatchFile(path.join(protoDir, file));
        }
      }
    },
    hotUpdate({ file, server }) {
      if (file.endsWith(".proto")) {
        console.log(`Proto file changed: ${file}. Rebuilding...`);

        try {
          execSync("pnpm run build:protoc", { stdio: "inherit" });
          console.log("Successfully rebuilt protos");
          server.hot.send({ type: "full-reload", path: "*" });
        } catch (error) {
          console.error("Failed to rebuild protos:", error);
        }
      }
    },
  };
}

export default defineConfig({
  resolve: {
    alias: {
      "@": path.resolve(import.meta.dirname, "src"),
    },
  },
  plugins: [
    tailwindcss(),
    protocBuild(),
    tanstackRouter({
      target: "react",
      autoCodeSplitting: true,
    }),
    react(),
    babel({ presets: [reactCompilerPreset()] }),
  ],
});
