# 🌌 GraviPop: Stellar Conservatory

> **A high-performance hybrid-casual cosmic merge puzzle game built in pure Rust.**
> Zero garbage collection pauses, instant sub-second cold starts, zero engine bloat, and universal compatibility across Desktop, Mobile, and Web.

---

## 🚀 Recent Accomplishments & System Upgrades

- **Physics & Merge Engine Overhaul (100% Resolved)**:
  - **Immediate Surface Touch Merging**: Replaced the previous 8% interpenetration threshold (`dist < contact * 0.92`) with a robust surface contact detector (`dist <= contact * 1.02`). Same-tier celestial bodies now fuse reliably upon touching, whether falling, rolling, or resting side-by-side.
  - **Unstable Vertical Equilibrium Break (Totem Pole Fix)**: When celestial spheres land vertically atop one another (`dx ≈ 0`), the engine applies a lateral slope perturbation to the collision normal and imparts rolling velocity. Spheres realistically slide and roll down curved shoulders into resting crevices, mimicking genuine physical spheres.
  - **Zero Floor & Wall Tunneling**: Implemented post-solver boundary clamping (`JAR_LEFT`, `JAR_RIGHT`, `JAR_BOTTOM`) for all bodies. Settled bodies at the bottom of the container are held firmly above the container line without a single pixel protruding.
  - **Micro-Velocity Sleep**: Added resting velocity damping to prevent jitter at the bottom of heavy stacks.

- **Celestial Tier Visual Redesign (10 Unique Visual Identities)**:
  - **Asteroid**: Craggy dark basalt rock with 3 distinct crater pits and molten amber mineral veins.
  - **Moon**: Silvery regolith with dark lunar maria basalt seas, rayed impact craters, and crisp terminator rim lighting.
  - **Earth (Terrestrial)**: Deep azure oceans, emerald continental landmasses, polar ice caps, and dynamic swirling atmospheric cloud spirals.
  - **Gas Giant (Jupiter)**: 5 alternating horizontal turbulent storm belts and an elliptical swirling **Great Red Spot** storm vortex.
  - **Ringed Giant (Saturn)**: Golden sphere with **3D perspective dual-ring system** featuring depth occlusion (back ring behind, front ring in front) and planetary shadow.
  - **Ice Giant**: Crystalline turquoise glacial facets, geometric ice plates, and glowing neon-cyan auroral crowns at magnetic poles.
  - **Red Dwarf**: Convective boiling solar granules and dynamic arching coronal prominences / pulsating solar flares.
  - **Blue Supergiant**: Blinding white-hot thermonuclear core with 8 radiant cardinal starburst light rays and plasma filaments.
  - **Pulsar (Magnetar)**: Ultra-dense violet neutron core with equatorial magnetic flux loops and **dual rotating relativistic radiation jets**.
  - **Singularity**: Absolute pitch-black event horizon encircled by an **Einstein gravitational lensing photon ring** and an iridescent violet/gold relativistic accretion disk with Doppler boosting.

- **Deconflicted UI Miniature Previews**:
  - Implemented `BodyRenderer::draw_preview(tier, center, radius: 22.0)`. Choice A, Choice B, and Comet Reserve slots now render neat, compact celestial miniatures that never spill over card borders or obscure card labels.

- **Universal Cross-Ecosystem Normalization**:
  - Implemented dynamic virtual camera projection (`720 × 1280`) with automatic pillarbox/letterbox scaling. Preserves sharp aspect ratio and prevents any visual distortion across 16:9, 19.5:9, 21:9 mobile displays and high-DPI desktop monitors.
  - Unified input system supporting mouse clicks, multi-touch drag-to-aim with release-to-drop (`TouchPhase`), and full desktop keyboard controls (`Arrow Keys / A / D / Spacebar`).
- **Zero-Glyph Procedural Vector Icon Architecture**:
  - Replaced standard unicode/emoji text elements with math-based vector rendering (`draw_vector_star`, `draw_vector_gem`, `draw_vector_play`, `draw_vector_pause`, `draw_vector_lock`, `draw_vector_close`).
- **Zero-Crash Android Native Integration**:
  - Configured miniquad JNI entry points (`quad_main`) and verified native compilation for `aarch64-linux-android`. All 18 automated unit tests pass with zero warnings.

---

## 🎮 Game Controls Across Ecosystems

| Action | Mobile (Touchscreen) | Desktop (Mouse & Keyboard) |
| :--- | :--- | :--- |
| **Aim Celestial Body** | Touch & slide finger across the jar | Move mouse or press `←` / `→` or `A` / `D` |
| **Drop Body** | Release finger from screen | Release mouse button or press `Space` / `↓` |
| **Swap Comet Reserve** | Tap center circular "RESERVE" slot | Click slot or press `R` / `B` |
| **Trigger Gravity Wave** | Tap "GRAVITY WAVE" button | Click button or press `1` |
| **Trigger Solar Flare** | Tap "SOLAR FLARE" button | Click button or press `2` |
| **Pause / Resume** | Tap top-right pause icon | Click icon or press `P` / `Esc` |

---

## 🛠️ Prerequisites & Installation

### Desktop (Windows, macOS, Linux)
1. **Rust Toolchain**: Install Rust (stable 1.70+) via [rustup.rs](https://rustup.rs/):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **Clone the Repository**:
   ```bash
   git clone https://github.com/areeb-azhar/gravipop-mobile.git
   cd gravipop-mobile
   ```
3. **Run Locally**:
   ```bash
   cargo run --release
   ```
4. **Run Unit & Physics Tests**:
   ```bash
   cargo test
   ```

---

## 📱 Mobile APK Build Command (Android)

You do **not** need to assemble custom Gradle projects manually. GraviPop uses `cargo-quad-apk` for 1-step Android builds:

### Prerequisites & Environment Setup
- Android SDK: `C:\Users\areeb\AppData\Local\Android\Sdk`
- Android NDK: `C:\Users\areeb\AppData\Local\Android\Sdk\ndk\28.2.13676358`
- `cargo-quad-apk` installed:
  ```bash
  cargo install cargo-quad-apk
  ```
- Android target added to Rust:
  ```bash
  rustup target add aarch64-linux-android
  ```

### The Exact Command to Build the APK
In your PowerShell terminal, set the NDK environment path and build:

```powershell
$env:NDK_HOME = "C:\Users\areeb\AppData\Local\Android\Sdk\ndk\28.2.13676358"
$env:ANDROID_HOME = "C:\Users\areeb\AppData\Local\Android\Sdk"
cargo quad-apk build --release
```

> **Tip (Permanent Setup)**: To set it once permanently so you never have to type `$env:NDK_HOME` again:
> ```powershell
> [Environment]::SetEnvironmentVariable("NDK_HOME", "C:\Users\areeb\AppData\Local\Android\Sdk\ndk\28.2.13676358", "User")
> [Environment]::SetEnvironmentVariable("ANDROID_HOME", "C:\Users\areeb\AppData\Local\Android\Sdk", "User")
> ```

The resulting signed APK will be generated at:
```text
target/android-artifacts/release/apk/gravipop-mobile.apk
```

### Install Directly to Connected Phone:
```bash
cargo quad-apk run --release
```
*(Or transfer `gravipop-mobile.apk` to your device and tap to install).*

---

## 🌐 WebAssembly (WASM) Build Command

To compile GraviPop for web browsers:
```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
```

---

## 🏛️ Project Architecture & File Hierarchy

```text
gravipop-mobile/
├── assets/
│   ├── font.ttf                        # High-legibility TrueType typography
│   ├── celestial_atlas.jpg             # High-res celestial planet art atlas
│   └── gravipop_icon.jpg               # Premium cosmic app icon artwork
├── src/
│   ├── main.rs                         # Desktop executable entry point
│   ├── lib.rs                          # Universal game loop, input normalization, quad_main
│   ├── core/
│   │   ├── config.rs                   # Virtual canvas, physical dimensions, deconflicted layout
│   │   ├── game_state.rs               # State machine (MainMenu, GalaxyMap, Playing, Paused, etc.)
│   │   ├── save_system.rs              # Player progression, stardust economy, sector stars
│   │   ├── sector.rs                   # 15 authored sectors across 3 chapters, objective tracker
│   │   └── mod.rs
│   ├── physics/
│   │   ├── celestial_tier.rs           # 10 cosmic tiers (Asteroid → Singularity)
│   │   ├── body.rs                     # Verlet/Euler integration, damping, velocities
│   │   ├── collision.rs                # Multi-pass impulse resolution & merge fusion logic
│   │   └── mod.rs
│   ├── graphics/
│   │   ├── icons.rs                    # Procedural vector drawing (stars, gems, locks, arrows)
│   │   ├── renderer.rs                 # Body aura, rings, atmospheric glow shaders
│   │   ├── particles.rs                # Cosmic dust bursts, fusion sparks, floating score text
│   │   ├── starfield.rs                # Parallax star layers and danger red-shift pulse
│   │   └── mod.rs
│   ├── audio/
│   │   ├── sound_synthesizer.rs        # In-memory WAV harmonic bell synthesis & victory chimes
│   │   └── mod.rs
│   ├── monetization/
│   │   ├── ads.rs                      # Rewarded & Interstitial ad state handlers
│   │   ├── billing.rs                  # IAP billing verification (Remove Ads, Star Pass)
│   │   ├── economy.rs                  # Catalog of items, packs, and cosmic skins
│   │   └── mod.rs
│   └── ui/
│       ├── hud.rs                      # Objective banner, score, danger alert
│       ├── game_over_modal.rs          # Overflow summary, ad revival, retry triggers
│       ├── shop_modal.rs               # Cosmic store modal with skins & IAP
│       ├── mock_ad_overlay.rs          # Ad viewing simulation overlay
│       └── mod.rs
├── Cargo.toml                          # Dependency manifest & Android metadata
└── gravipop_save.json                  # Local persistence save data
```

---

## 📸 Screenshots & Verification

> [!NOTE]
> Please capture and provide screenshots of:
> 1. **Main Menu**: Highlighting the title, best score, vector stardust crystal, and clean buttons.
> 2. **Galaxy Map**: Demonstrating the 3 Chapter tabs (*Nebula*, *Frost*, *Void*) and gold vector stars.
> 3. **Active Gameplay**: Showing the deconflicted top HUD, spawner bar, aim trajectory laser, and bottom ability controls.
