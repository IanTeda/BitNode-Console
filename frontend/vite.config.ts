import { defineConfig } from "vite";
import path from "path";
import babel from "@rolldown/plugin-babel";
import react, { reactCompilerPreset } from "@vitejs/plugin-react";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import tailwindcss from "@tailwindcss/vite";
import { protocBuild } from "./vite-plugins/protoc-build";

export default defineConfig({
  envPrefix: "BITNODE_",
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  plugins: [
    // Tanstack Router needs to be before the react plugin call, so it can intercept the react elements.
    // Let's put it at the start of the plugins array.
    tanstackRouter({
      target: "react",
      autoCodeSplitting: true,
    }),
    protocBuild({
      protoDir: path.resolve(__dirname, "protos"),
      outputDir: path.resolve(__dirname, "src/lib/generated_protos"),
    }),
    tailwindcss(),
    react(),
    babel({ presets: [reactCompilerPreset()] }),
  ],
});
