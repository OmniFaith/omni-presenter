import { $ } from "bun";
import { resolve } from "path";

const root = resolve(import.meta.dir, "..");
const srcDir = resolve(import.meta.dir, "src");
const outDir = resolve(root, "assets/web");

async function run() {
  console.log("Compiling TypeScript to assets/web/...");
  const tsc = resolve(import.meta.dir, "node_modules/typescript/bin/tsc");
  if (!(await Bun.file(tsc).exists())) {
    throw new Error("Missing locked TypeScript dependency. Run bun install --frozen-lockfile in web/.");
  }
  await $`bun ${tsc} --rootDir src --outDir ${outDir}`;
  await $`rm -rf ${outDir}/types`;

  console.log("Copying HTML and CSS files to assets/web/...");
  await $`mkdir -p ${outDir}/control ${outDir}/viewer ${outDir}/chat`;
  await $`cp ${srcDir}/control/*.html ${srcDir}/control/*.css ${outDir}/control/`;
  await $`cp ${srcDir}/viewer/*.html ${outDir}/viewer/`;
  await $`cp ${srcDir}/chat/*.html ${srcDir}/chat/*.css ${outDir}/chat/`;

  // Classic scripts in chat.html should not have `export {};` appended by tsc
  const chatJsFiles = await Array.fromAsync(new Bun.Glob("chat/*.js").scan({ cwd: outDir }));
  for (const file of chatJsFiles) {
    const fullPath = resolve(outDir, file);
    const content = await Bun.file(fullPath).text();
    const cleaned = content.replace(/\n?export\s*\{\s*\};?\s*$/g, "");
    if (cleaned !== content) {
      await Bun.write(fullPath, cleaned);
    }
  }

  console.log("Web assets built successfully to assets/web/");
}

run().catch((err) => {
  console.error("Build failed:", err);
  process.exit(1);
});
