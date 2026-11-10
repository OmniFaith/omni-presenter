import { build } from "bun";
import { resolve } from "path";
const root = resolve(import.meta.dir, "..");
const outDir = resolve(root, "assets/web");
async function run() {
    console.log("Building TypeScript bundles for web interfaces...");
    // Control panel
    await build({
        entrypoints: [resolve(import.meta.dir, "src/control/control.ts")],
        outdir: resolve(outDir, "control"),
        naming: "[name].js",
        target: "browser",
        minify: false,
    });
    // Viewer
    await build({
        entrypoints: [
            resolve(import.meta.dir, "src/viewer/viewer.ts"),
            resolve(import.meta.dir, "src/viewer/design.ts"),
        ],
        outdir: resolve(outDir, "viewer"),
        naming: "[name].js",
        target: "browser",
        minify: false,
    });
    console.log("Web bundles built successfully to assets/web/");
}
run().catch((err) => {
    console.error("Build failed:", err);
    process.exit(1);
});
