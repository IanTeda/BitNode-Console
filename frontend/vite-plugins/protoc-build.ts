import { exec } from "child_process";
import { existsSync, readdirSync } from "fs";
import { readFile, writeFile } from "fs/promises";
import { join } from "path";
import { promisify } from "util";
import type { Plugin } from "vite";

const execAsync = promisify(exec);

interface Options {
  protoDir: string;
  outputDir: string;
}

function collectFiles(dir: string, ext: string): string[] {
  if (!existsSync(dir)) return [];
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) return collectFiles(full, ext);
    if (entry.name.endsWith(ext)) return [full];
    return [];
  });
}

async function prependTsNoCheck(file: string): Promise<void> {
  const content = await readFile(file, "utf-8");
  if (!content.startsWith("// @ts-nocheck")) {
    await writeFile(file, `// @ts-nocheck\n${content}`);
  }
}

async function runProtoc(protoDir: string, outputDir: string): Promise<void> {
  const protoFiles = collectFiles(protoDir, ".proto");
  if (protoFiles.length === 0) return;

  const cmd = [
    "protoc",
    `--ts_out=${outputDir}`,
    `--ts_opt=long_type_string`,
    `--proto_path=${protoDir}`,
    ...protoFiles,
  ].join(" ");

  const { stdout, stderr } = await execAsync(cmd);
  if (stdout) console.log(`[protoc-build] ${stdout.trim()}`);
  if (stderr) console.warn(`[protoc-build] ${stderr.trim()}`);

  await Promise.all(collectFiles(outputDir, ".ts").map(prependTsNoCheck));
}

export function protocBuild({ protoDir, outputDir }: Options): Plugin {
  return {
    name: "protoc-build",
    async buildStart() {
      try {
        await runProtoc(protoDir, outputDir);
      } catch (err) {
        console.error("[protoc-build] build failed:", err);
      }
    },
    configureServer(server) {
      server.watcher.add(protoDir);
    },
    async handleHotUpdate({ file, server }) {
      if (!file.endsWith(".proto")) return;
      console.log(`[protoc-build] ${file} changed, rebuilding...`);
      try {
        await runProtoc(protoDir, outputDir);
        console.log("[protoc-build] rebuild complete");
      } catch (err) {
        console.error("[protoc-build] rebuild failed:", err);
      }
      server.ws.send({ type: "full-reload", path: "*" });
    },
  };
}
