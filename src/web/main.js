import { inject } from "@vercel/analytics";
import { CELESTIAL_TIERS } from "./codex.js";
import { app as firebaseApp } from "./firebase.js";

inject();

const defaultLeaderboardUrl =
  typeof window !== "undefined" &&
  (window.location.hostname.includes("web.app") || window.location.hostname.includes("firebaseapp.com"))
    ? "https://graviity-zeta.vercel.app/api/leaderboard"
    : "/api/leaderboard";

window.GRAVIPOP_LEADERBOARD_URL = import.meta.env.VITE_LEADERBOARD_URL || defaultLeaderboardUrl;

window.GravipopAds?.configure({
  mode: import.meta.env.VITE_ADS_MODE || "test",
  rewarded: import.meta.env.VITE_GOOGLE_AD_REWARDED_UNIT || "",
  interstitial: import.meta.env.VITE_GOOGLE_AD_INTERSTITIAL_UNIT || "",
  left: import.meta.env.VITE_GOOGLE_AD_SIDEBAR_LEFT_UNIT || "",
  right: import.meta.env.VITE_GOOGLE_AD_SIDEBAR_RIGHT_UNIT || "",
});

// Populate Codex list with photos
const codexContainer = document.getElementById("codex-list");
if (codexContainer) {
  codexContainer.innerHTML = CELESTIAL_TIERS.map(
    (tier) => `
    <div class="codex-card" style="--tier-glow: ${tier.glow}">
      <div class="card-thumb-wrap">
        <img src="${tier.image || '/assets/planets/tier_' + tier.tier + '.jpg'}" alt="${tier.name}" class="planet-thumb" loading="lazy" />
      </div>
      <div class="card-body">
        <div class="card-top">
          <span class="tier-dot" style="background-color: ${tier.color}; box-shadow: 0 0 8px ${tier.glow};"></span>
          <span class="tier-name">${tier.name}</span>
          <span class="tier-pts" title="Points are awarded for merging, never for dropping a planet">${tier.points}</span>
        </div>
        <p class="card-desc">${tier.desc}</p>
      </div>
    </div>
  `
  ).join("");
}

// Ensure Canvas is focused on click
const canvas = document.getElementById("glcanvas");
if (canvas) {
  canvas.addEventListener("pointerdown", () => {
    canvas.focus();
  });
}

// Fullscreen API helper
function toggleFullscreen() {
  if (!document.fullscreenElement) {
    document.documentElement.requestFullscreen().catch(() => {});
  } else {
    document.exitFullscreen().catch(() => {});
  }
}

let lastWebClickTime = 0;
function debouncedClick(fn, delay = 350) {
  return function (e) {
    const now = Date.now();
    if (now - lastWebClickTime < delay) {
      if (e?.preventDefault) e.preventDefault();
      return;
    }
    lastWebClickTime = now;
    return fn.apply(this, arguments);
  };
}

const fullscreenBtn = document.getElementById("btn-fullscreen");
if (fullscreenBtn) {
  fullscreenBtn.addEventListener("click", debouncedClick(toggleFullscreen));
}

const mobileFullscreenBtn = document.getElementById("btn-mobile-fullscreen");
if (mobileFullscreenBtn) {
  mobileFullscreenBtn.addEventListener("click", debouncedClick(toggleFullscreen));
}

// Focus / Maximize Game Mode (collapses side panels to maximize canvas)
const leftPanel = document.getElementById("left-panel");
const rightPanel = document.getElementById("right-panel");
const focusBtn = document.getElementById("btn-focus");

function toggleFocusMode() {
  const isCollapsed = leftPanel?.classList.contains("collapsed");
  if (isCollapsed) {
    leftPanel?.classList.remove("collapsed");
    rightPanel?.classList.remove("collapsed");
    if (focusBtn) focusBtn.textContent = "↔";
  } else {
    leftPanel?.classList.add("collapsed");
    rightPanel?.classList.add("collapsed");
    if (focusBtn) focusBtn.textContent = "🗗";
  }
  // Notify Macroquad to resize canvas to the new width
  setTimeout(() => {
    window.dispatchEvent(new Event("resize"));
  }, 100);
}

if (focusBtn) {
  focusBtn.addEventListener("click", debouncedClick(toggleFocusMode));
}

// Global Hotkeys: 'F' = Fullscreen, 'M' = Maximize / Focus
window.addEventListener("keydown", (e) => {
  if (document.activeElement?.tagName === "INPUT") return;

  if (e.key === "f" || e.key === "F") {
    toggleFullscreen();
  } else if (e.key === "m" || e.key === "M") {
    toggleFocusMode();
  }
});

// Live Stats Updater (stardust, high score from localStorage)
function updateLiveStats() {
  try {
    const raw = localStorage.getItem("gravipop.save.v1");
    if (!raw) return;
    const save = JSON.parse(raw);
    const stardustEl = document.getElementById("stat-stardust");
    const highScoreEl = document.getElementById("stat-high-score");
    if (stardustEl && typeof save.stardust === "number") {
      stardustEl.textContent = save.stardust.toLocaleString();
    }
    if (highScoreEl && typeof save.high_score === "number") {
      highScoreEl.textContent = save.high_score.toLocaleString();
    }
  } catch (_) {}
}

setInterval(updateLiveStats, 1000);
updateLiveStats();

// Mobile Drawer Controls
const mobileCodexBtn = document.getElementById("btn-mobile-codex");
const mobileControlsBtn = document.getElementById("btn-mobile-controls");
const closeLeftBtn = document.getElementById("btn-close-left");
const closeRightBtn = document.getElementById("btn-close-right");
const drawerBackdrop = document.getElementById("drawer-backdrop");

function closeDrawers() {
  leftPanel?.classList.remove("open");
  rightPanel?.classList.remove("open");
  drawerBackdrop?.classList.remove("active");
}

if (mobileCodexBtn) {
  mobileCodexBtn.addEventListener(
    "click",
    debouncedClick(() => {
      rightPanel?.classList.remove("open");
      leftPanel?.classList.add("open");
      drawerBackdrop?.classList.add("active");
    })
  );
}

if (mobileControlsBtn) {
  mobileControlsBtn.addEventListener(
    "click",
    debouncedClick(() => {
      leftPanel?.classList.remove("open");
      rightPanel?.classList.add("open");
      drawerBackdrop?.classList.add("active");
    })
  );
}

if (closeLeftBtn) closeLeftBtn.addEventListener("click", debouncedClick(closeDrawers));
if (closeRightBtn) closeRightBtn.addEventListener("click", debouncedClick(closeDrawers));
if (drawerBackdrop) drawerBackdrop.addEventListener("click", debouncedClick(closeDrawers));

console.log("🌌 GraviPop Web Shell Ready: Canvas 100% Scaled & Mobile Optimized");
