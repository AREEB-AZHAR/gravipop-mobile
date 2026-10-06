import { cpSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
function run(command, args, env = process.env, capture = false) {
  const result = spawnSync(command, args, { cwd: root, env, encoding: "utf8", stdio: capture ? "pipe" : "inherit" });
  if (result.error || result.status !== 0) throw new Error(result.error?.message || result.stderr || `${command} failed (${result.status})`);
  return result.stdout;
}

// Use the Java host belonging to the exact Miniquad crate linked into Rust.
const metadata = JSON.parse(run("cargo", ["metadata", "--format-version", "1", "--filter-platform", "aarch64-linux-android"], process.env, true));
const miniquad = metadata.packages.find((pkg) => pkg.name === "miniquad");
if (!miniquad) throw new Error("Miniquad was not found in Cargo metadata.");
const javaRoot = join(dirname(miniquad.manifest_path), "java");
const runtimeDest = join(root, "android/app/src/main/java/com/gravipop/runtime");
const quadDest = join(root, "android/app/src/main/java/quad_native");
mkdirSync(runtimeDest, { recursive: true });
mkdirSync(quadDest, { recursive: true });
const host = readFileSync(join(javaRoot, "MainActivity.java"), "utf8")
  .replaceAll("TARGET_PACKAGE_NAME", "com.gravipop.runtime")
  .replaceAll("MainActivity", "QuadActivity")
  .replaceAll("LIBRARY_NAME", "gravipop_mobile");
writeFileSync(join(runtimeDest, "QuadActivity.java"), `// Miniquad ${miniquad.version} Android host (MIT OR Apache-2.0).\n${host.trimEnd()}\n`);
cpSync(join(javaRoot, "QuadNative.java"), join(quadDest, "QuadNative.java"));
const licenseDest = join(root, "android/third_party/miniquad");
mkdirSync(licenseDest, { recursive: true });
for (const name of ["LICENSE-MIT", "LICENSE-APACHE"]) {
  writeFileSync(join(licenseDest, name), readFileSync(join(dirname(miniquad.manifest_path), name), "utf8").trimEnd() + "\n");
}
if (process.argv.includes("--java-only")) process.exit(0);

const sdk = process.env.ANDROID_HOME || process.env.ANDROID_SDK_ROOT;
const ndk = process.env.NDK_HOME || (sdk ? join(sdk, "ndk/28.2.13676358") : "");
if (!ndk) throw new Error("Set ANDROID_HOME (Android SDK) and NDK_HOME (NDK 28.2.13676358).");
const hostPlatform = process.platform === "win32" ? "windows-x86_64" : process.platform === "darwin" ? "darwin-x86_64" : "linux-x86_64";
const toolchain = join(ndk, "toolchains/llvm/prebuilt", hostPlatform, "bin");
const extension = process.platform === "win32" ? ".cmd" : "";
const targets = {
  "arm64-v8a": ["aarch64-linux-android", "aarch64-linux-android24-clang"],
  "armeabi-v7a": ["armv7-linux-androideabi", "armv7a-linux-androideabi24-clang"],
  "x86_64": ["x86_64-linux-android", "x86_64-linux-android24-clang"],
};
const abis = (process.argv.find((arg) => arg.startsWith("--abis="))?.split("=")[1] || "arm64-v8a").split(",");
for (const abi of abis) {
  if (!targets[abi]) throw new Error(`Unsupported ABI: ${abi}`);
  const [target, clang] = targets[abi];
  const key = target.replaceAll("-", "_").toUpperCase();
  const env = { ...process.env, [`CARGO_TARGET_${key}_LINKER`]: join(toolchain, clang + extension),
    [`CC_${target.replaceAll("-", "_")}`]: join(toolchain, clang + extension),
    [`AR_${target.replaceAll("-", "_")}`]: join(toolchain, "llvm-ar" + (process.platform === "win32" ? ".exe" : "")),
    [`CARGO_TARGET_${key}_RUSTFLAGS`]: "-C link-arg=-Wl,-z,max-page-size=16384" };
  run("cargo", ["build", "--release", "--target", target, "--lib"], env);
  const destination = join(root, "android/app/src/main/jniLibs", abi);
  mkdirSync(destination, { recursive: true });
  cpSync(join(root, "target", target, "release/libgravipop_mobile.so"), join(destination, "libgravipop_mobile.so"));
}
console.log(`Native ads bridge and Miniquad host ready for ${abis.join(", ")}. Run Gradle assembleDebug from android/.`);
