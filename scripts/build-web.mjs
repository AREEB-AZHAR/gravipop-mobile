import { dirname, resolve } from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");

function run(command, args) {
  return new Promise((resolvePromise, reject) => {
    const child = spawn(command, args, { cwd: projectRoot, stdio: "inherit", shell: true });
    child.once("error", reject);
    child.once("exit", (code) =>
      code === 0 ? resolvePromise() : reject(new Error(`${command} exited with ${code}`))
    );
  });
}

console.log("==> Step 1: Building WebAssembly Release & Syncing Glue...");
await run("node", ["scripts/build-wasm.mjs"]);

console.log("==> Step 2: Building Production Vite Application Bundle...");
await run("npx", ["vite", "build"]);

console.log("==> GraviPop Web Release successfully built to ./dist!");
