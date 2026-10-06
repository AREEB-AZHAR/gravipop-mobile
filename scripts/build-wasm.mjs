import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const publicRoot = join(projectRoot, "public");
const webRoot = join(projectRoot, "web");
const cargoHome = process.env.CARGO_HOME ?? join(homedir(), ".cargo");

function run(command, args) {
  return new Promise((resolvePromise, reject) => {
    console.log(`Running: ${command} ${args.join(" ")}`);
    const child = spawn(command, args, { cwd: projectRoot, stdio: "inherit", shell: true });
    child.once("error", reject);
    child.once("exit", (code) =>
      code === 0 ? resolvePromise() : reject(new Error(`${command} exited with code ${code}`))
    );
  });
}

function findMacroquadBundle() {
  const registry = join(cargoHome, "registry", "src");
  if (!existsSync(registry)) return null;
  for (const source of readdirSync(registry, { withFileTypes: true })) {
    if (!source.isDirectory()) continue;
    const sourceDir = join(registry, source.name);
    for (const crate of readdirSync(sourceDir, { withFileTypes: true })) {
      if (!crate.isDirectory() || !crate.name.startsWith("macroquad-")) continue;
      const bundle = join(sourceDir, crate.name, "js", "mq_js_bundle.js");
      if (existsSync(bundle)) return bundle;
    }
  }
  return null;
}

console.log("==> Building GraviPop WebAssembly release binary...");
await run("cargo", ["build", "--release", "--target", "wasm32-unknown-unknown", "--bin", "gravipop-mobile"]);

const wasmSource = join(projectRoot, "target", "wasm32-unknown-unknown", "release", "gravipop-mobile.wasm");
if (!existsSync(wasmSource)) {
  throw new Error(`WASM binary not found at ${wasmSource}`);
}

// Find and patch Macroquad bundle for strict mode compatibility
const bundleSource = findMacroquadBundle();
let bundleContent = "";
if (bundleSource && existsSync(bundleSource)) {
  bundleContent = readFileSync(bundleSource, "utf8");
} else if (existsSync(join(webRoot, "mq_js_bundle.js"))) {
  bundleContent = readFileSync(join(webRoot, "mq_js_bundle.js"), "utf8");
} else {
  throw new Error("Could not find Macroquad's mq_js_bundle.js.");
}

// Apply strict mode & high-DPI sharpness fixes
bundleContent = bundleContent
  .replace("register_plugin=function(e)", "window.register_plugin=function(e)")
  .replace("function dpi_scale(){return high_dpi?window.devicePixelRatio||1:1}", "function dpi_scale(){return window.devicePixelRatio||1}")
  .replace(
    'wasm_exports.touch(SAPP_EVENTTYPE_TOUCHES_BEGAN,t.identifier,n.x,n.y)}})',
    'wasm_exports.touch(SAPP_EVENTTYPE_TOUCHES_BEGAN,t.identifier,n.x,n.y),wasm_exports.mouse_move(Math.floor(n.x),Math.floor(n.y)),wasm_exports.mouse_down(n.x,n.y,0)}})'
  )
  .replace(
    'wasm_exports.touch(SAPP_EVENTTYPE_TOUCHES_MOVED,t.identifier,n.x,n.y)}})',
    'wasm_exports.touch(SAPP_EVENTTYPE_TOUCHES_MOVED,t.identifier,n.x,n.y),wasm_exports.mouse_move(Math.floor(n.x),Math.floor(n.y))}})'
  )
  .replace(
    'wasm_exports.touch(SAPP_EVENTTYPE_TOUCHES_ENDED,t.identifier,n.x,n.y)}})',
    'wasm_exports.touch(SAPP_EVENTTYPE_TOUCHES_ENDED,t.identifier,n.x,n.y),wasm_exports.mouse_up(n.x,n.y,0)}})'
  )
  .replace(
    'wasm_exports.touch(SAPP_EVENTTYPE_TOUCHES_CANCELED,t.identifier,n.x,n.y)}})',
    'wasm_exports.touch(SAPP_EVENTTYPE_TOUCHES_CANCELED,t.identifier,n.x,n.y),wasm_exports.mouse_up(n.x,n.y,0)}})'
  );

// Sync destinations
for (const dir of [publicRoot, webRoot]) {
  mkdirSync(join(dir, "assets"), { recursive: true });
  cpSync(wasmSource, join(dir, "gravipop-mobile.wasm"));
  cpSync(join(projectRoot, "assets"), join(dir, "assets"), { recursive: true });
  writeFileSync(join(dir, "mq_js_bundle.js"), bundleContent, "utf8");
  if (existsSync(join(webRoot, "gravipop_web.js")) && dir !== webRoot) {
    cpSync(join(webRoot, "gravipop_web.js"), join(dir, "gravipop_web.js"));
  }
  if (dir !== webRoot) {
    cpSync(join(webRoot, "gravipop_ads.js"), join(dir, "gravipop_ads.js"));
  }
  if (existsSync(join(webRoot, "env.js")) && dir !== webRoot) {
    cpSync(join(webRoot, "env.js"), join(dir, "env.js"));
  }
}

console.log("==> WASM build ready in ./public and ./web!");
