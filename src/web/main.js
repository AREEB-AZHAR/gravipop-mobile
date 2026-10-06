import { CELESTIAL_TIERS } from "./codex.js";

// Populate Codex list
const codexContainer = document.getElementById("codex-list");
if (codexContainer) {
  codexContainer.innerHTML = CELESTIAL_TIERS.map(
    (tier) => `
    <div class="codex-card" style="--tier-glow: ${tier.glow}">
      <div class="card-top">
        <span class="tier-dot" style="background-color: ${tier.color}; box-shadow: 0 0 8px ${tier.glow};"></span>
        <span class="tier-name">${tier.name}</span>
        <span class="tier-pts">${tier.points}</span>
      </div>
      <p class="card-desc">${tier.desc}</p>
    </div>
  `
  ).join("");
}

// Fullscreen Toggle
const fullscreenBtn = document.getElementById("btn-fullscreen");
if (fullscreenBtn) {
  fullscreenBtn.addEventListener("click", () => {
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen().catch(() => {});
    } else {
      document.exitFullscreen().catch(() => {});
    }
  });
}

// Global hotkey 'F' for fullscreen
window.addEventListener("keydown", (e) => {
  if (e.key === "f" || e.key === "F") {
    if (document.activeElement?.tagName !== "INPUT") {
      if (!document.fullscreenElement) {
        document.documentElement.requestFullscreen().catch(() => {});
      } else {
        document.exitFullscreen().catch(() => {});
      }
    }
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

// Mobile Drawer Toggles
const mobileCodexBtn = document.getElementById("btn-mobile-codex");
const mobileControlsBtn = document.getElementById("btn-mobile-controls");
const leftPanel = document.getElementById("left-panel");
const rightPanel = document.getElementById("right-panel");

if (mobileCodexBtn && leftPanel) {
  mobileCodexBtn.addEventListener("click", () => {
    const isVisible = leftPanel.style.display === "flex";
    leftPanel.style.display = isVisible ? "none" : "flex";
    leftPanel.style.position = "absolute";
    leftPanel.style.left = "0";
    leftPanel.style.top = "56px";
    leftPanel.style.zIndex = "40";
    if (!isVisible && rightPanel) rightPanel.style.display = "none";
  });
}

if (mobileControlsBtn && rightPanel) {
  mobileControlsBtn.addEventListener("click", () => {
    const isVisible = rightPanel.style.display === "flex";
    rightPanel.style.display = isVisible ? "none" : "flex";
    rightPanel.style.position = "absolute";
    rightPanel.style.right = "0";
    rightPanel.style.top = "56px";
    rightPanel.style.zIndex = "40";
    if (!isVisible && leftPanel) leftPanel.style.display = "none";
  });
}

console.log("🌌 GraviPop Web Shell Initialized");
