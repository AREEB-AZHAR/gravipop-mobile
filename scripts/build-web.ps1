$ErrorActionPreference = "Stop"

if (-not $env:GRAVIPOP_SUPABASE_URL -or -not $env:GRAVIPOP_SUPABASE_ANON_KEY) {
    Write-Host "Note: GRAVIPOP_SUPABASE_URL or GRAVIPOP_SUPABASE_ANON_KEY not set. Building in local/offline mode."
}

Write-Host "==> Compiling GraviPop WebAssembly release..."
cargo build --release --target wasm32-unknown-unknown --bin gravipop-desktop

New-Item -ItemType Directory -Force -Path "public\assets" | Out-Null
New-Item -ItemType Directory -Force -Path "web\assets" | Out-Null

$wasmOutput = if (Test-Path "target\wasm32-unknown-unknown\release\gravipop-desktop.wasm") { "target\wasm32-unknown-unknown\release\gravipop-desktop.wasm" } else { "target\wasm32-unknown-unknown\release\gravipop_desktop.wasm" }
Copy-Item $wasmOutput "public\gravipop-mobile.wasm" -Force
Copy-Item $wasmOutput "web\gravipop-mobile.wasm" -Force
Copy-Item "assets\*" "public\assets" -Recurse -Force
Copy-Item "assets\*" "web\assets" -Recurse -Force

$bundle = Get-ChildItem "$env:USERPROFILE\.cargo\registry\src" -Recurse -Filter "mq_js_bundle.js" |
    Where-Object { $_.FullName -match "macroquad-" } |
    Select-Object -First 1

if ($bundle) {
    $bundleContent = Get-Content $bundle.FullName -Raw
    $bundleContent = $bundleContent.Replace("register_plugin=function(e)", "window.register_plugin=function(e)")
    Set-Content "public\mq_js_bundle.js" -Value $bundleContent -NoNewline
    Set-Content "web\mq_js_bundle.js" -Value $bundleContent -NoNewline
}

Write-Host "==> Building production Vite web bundle..."
npx vite build

Write-Host "Web release is ready in .\dist and .\public. Run 'npm run dev' or 'npm run preview'."
