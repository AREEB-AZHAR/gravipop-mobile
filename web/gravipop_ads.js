// Real Google Publisher Tag ads. Official Google demo inventory is used only
// in test mode; live mode requires the publisher's own Ad Manager unit paths.
"use strict";

(function () {
    const TEST_UNITS = Object.freeze({
        rewarded: "/22639388115/rewarded_web_example",
        interstitial: "/6355419/Travel/Europe/France/Paris",
        left: "/6355419/Travel/Europe",
        right: "/6355419/Travel/Europe",
    });
    const formats = { rewarded: "REWARDED", interstitial: "GAME_MANUAL_INTERSTITIAL" };
    const fullscreen = { rewarded: null, interstitial: null };
    const sidebars = new Map();
    const results = new Map();
    let config = null;
    let initialized = false;
    let sdkReady = false;
    let sdkFailed = false;
    let removed = false;
    let active = null;
    let retryTimer;
    let sdkTimer;
    let resizeTimer;

    function setStatus(message) {
        const el = document.getElementById("ads-status");
        if (el) el.textContent = message;
    }

    function available(kind) {
        return Boolean(sdkReady && !active && fullscreen[kind]?.ready
            && (kind === "rewarded" || !removed));
    }

    function railLayout() {
        if (!config) return [];
        const arena = document.getElementById("canvas-wrapper");
        if (!arena) return [];
        const bounds = arena.getBoundingClientRect();
        // The canvas letterboxes the 720 x 1280 game. Ads occupy only those
        // empty margins, with 16 px clearance on each side of an ad rail.
        const gameWidth = Math.min(bounds.width / 720, bounds.height / 1280) * 720;
        const width = (bounds.width - gameWidth) / 2 - 32;
        const height = bounds.height - 72;
        const sizes = [[300, 600], [160, 600], [300, 250], [250, 250], [200, 200]]
            .filter(([w, h]) => w <= width && h <= height);
        const visible = window.innerWidth > 1080 && !removed && !sdkFailed && sizes.length > 0
            && (config?.mode === "test" || config?.mode === "live");
        return ["left", "right"].map((side) => {
            const rail = document.getElementById(`ad-rail-${side}`);
            if (rail) {
                rail.hidden = !visible || !config[side];
                rail.style.width = `${Math.max(0, width)}px`;
                const label = rail.querySelector(".ad-label");
                if (label) label.textContent = config.mode === "test" ? "TEST ADVERTISEMENT" : "ADVERTISEMENT";
            }
            return { side, rail, sizes: visible ? sizes : [] };
        });
    }

    function destroyFullscreen(kind) {
        const record = fullscreen[kind];
        if (!record) return;
        clearTimeout(record.timer);
        fullscreen[kind] = null;
        window.googletag.destroySlots([record.slot]);
    }

    function scheduleReload(delay = 30000) {
        if (retryTimer) return;
        retryTimer = setTimeout(() => {
            retryTimer = null;
            if (sdkReady && !active) {
                loadFullscreen("rewarded");
                if (!removed) loadFullscreen("interstitial");
            }
        }, delay);
    }

    function loadFullscreen(kind) {
        if (!config[kind] || fullscreen[kind] || active || (kind === "interstitial" && removed)) return;
        const tag = window.googletag;
        const slot = tag.defineOutOfPageSlot(config[kind], tag.enums.OutOfPageFormat[formats[kind]]);
        if (!slot) return; // Unsupported device/format never blocks the game.
        slot.addService(tag.pubads());
        const record = { slot, ready: null, loadedAt: 0, timer: null };
        fullscreen[kind] = record;
        // A blocked request, no-fill, or SDK error must not leave a stale slot.
        record.timer = setTimeout(() => {
            if (fullscreen[kind] === record && !record.ready) {
                destroyFullscreen(kind);
                scheduleReload();
            }
        }, 25000);
        tag.display(slot);
    }

    function renderSidebars() {
        if (!sdkReady) { railLayout(); return; }
        for (const { side, rail, sizes } of railLayout()) {
            if (!rail) continue;
            const previous = sidebars.get(side);
            const key = JSON.stringify(sizes);
            if (previous?.key === key && previous.empty) { rail.hidden = true; continue; }
            if (rail.hidden || previous?.key !== key) {
                if (previous) window.googletag.destroySlots([previous.slot]);
                sidebars.delete(side);
                const container = document.getElementById(`google-ad-${side}`);
                if (container) container.replaceChildren();
            }
            if (rail.hidden || sidebars.has(side)) continue;
            const slot = window.googletag.defineSlot(config[side], sizes, `google-ad-${side}`);
            if (!slot) { rail.hidden = true; continue; }
            slot.addService(window.googletag.pubads());
            sidebars.set(side, { slot, key });
            const container = document.getElementById(`google-ad-${side}`);
            if (container) container.style.minHeight = `${Math.max(...sizes.map((size) => size[1]))}px`;
            window.googletag.display(slot);
        }
    }

    function complete(slot, result) {
        if (!active || active.slot !== slot) return;
        const request = active;
        active = null;
        // Store exactly one terminal result, identified by the Rust request.
        results.set(request.id, result);
        destroyFullscreen(request.kind);
        setStatus(config.mode === "test" ? "Google test ads" : "Ads by Google");
        document.getElementById("glcanvas")?.focus({ preventScroll: true });
        scheduleReload(1000);
    }

    function initializeSdk() {
        if (sdkReady || config.mode === "off") return;
        clearTimeout(sdkTimer);
        sdkReady = true;
        sdkFailed = false;
        const pubads = window.googletag.pubads();
        pubads.addEventListener("rewardedSlotReady", (event) => {
            const record = fullscreen.rewarded;
            if (record?.slot === event.slot) {
                clearTimeout(record.timer);
                record.ready = () => event.makeRewardedVisible();
                record.loadedAt = Date.now();
            }
        });
        pubads.addEventListener("rewardedSlotGranted", (event) => {
            if (active?.slot === event.slot && active.kind === "rewarded") active.earned = true;
        });
        // Video completion alone is deliberately NOT used to grant rewards.
        pubads.addEventListener("rewardedSlotClosed", (event) => {
            complete(event.slot, active?.earned ? 1 : 2);
        });
        pubads.addEventListener("gameManualInterstitialSlotReady", (event) => {
            const record = fullscreen.interstitial;
            if (record?.slot === event.slot) {
                clearTimeout(record.timer);
                record.ready = () => event.makeGameManualInterstitialVisible();
                record.loadedAt = Date.now();
            }
        });
        pubads.addEventListener("gameManualInterstitialSlotClosed", (event) => complete(event.slot, 2));
        pubads.addEventListener("slotRenderEnded", (event) => {
            for (const [side, record] of sidebars) {
                if (record.slot === event.slot && event.isEmpty) {
                    record.empty = true;
                    document.getElementById(`ad-rail-${side}`).hidden = true;
                }
            }
            for (const kind of Object.keys(fullscreen)) {
                if (fullscreen[kind]?.slot === event.slot && event.isEmpty) {
                    if (active?.slot === event.slot) complete(event.slot, 3);
                    else destroyFullscreen(kind);
                    scheduleReload();
                }
            }
        });
        // Sidebar DOM already exists before services and ad requests start.
        window.googletag.enableServices();
        renderSidebars();
        loadFullscreen("rewarded");
        if (!removed) loadFullscreen("interstitial");
        setStatus(config.mode === "test" ? "Google test ads" : "Ads by Google");
    }

    function failedSdk() {
        if (sdkReady) return;
        sdkFailed = true;
        clearTimeout(sdkTimer);
        setStatus("Ads unavailable");
        for (const side of ["left", "right"]) {
            const rail = document.getElementById(`ad-rail-${side}`);
            if (rail) rail.hidden = true;
        }
    }

    function configure(options) {
        if (initialized) return;
        initialized = true;
        const mode = ["test", "live", "off"].includes(options.mode) ? options.mode : "off";
        config = { ...options, mode, ...(mode === "test" ? TEST_UNITS : {}) };
        for (const kind of ["rewarded", "interstitial", "left", "right"]) {
            // A live build must never silently fall back to somebody else's inventory.
            if (mode !== "test" && !/^\/\d+\/[A-Za-z0-9_./-]+$/.test(config[kind] || "")) config[kind] = "";
        }
        try { removed = Boolean(JSON.parse(localStorage.getItem("gravipop.save.v1") || "{}").ads_removed); } catch (_) {}
        if (mode === "off") { setStatus("Ads disabled"); return; }
        railLayout();
        window.googletag = window.googletag || { cmd: [] };
        window.googletag.cmd.push(initializeSdk);
        const script = document.createElement("script");
        script.async = true;
        script.crossOrigin = "anonymous";
        script.src = "https://securepubads.g.doubleclick.net/tag/js/gpt.js";
        script.onerror = failedSdk;
        sdkTimer = setTimeout(failedSdk, 15000);
        document.head.appendChild(script);
        window.addEventListener("resize", () => {
            clearTimeout(resizeTimer);
            resizeTimer = setTimeout(renderSidebars, 250);
        });
    }

    window.GravipopAds = Object.freeze({
        configure,
        isReady(rewarded) { return available(rewarded ? "rewarded" : "interstitial"); },
        show(requestId, rewarded) {
            const kind = rewarded ? "rewarded" : "interstitial";
            if (!available(kind) || !Number.isInteger(requestId) || requestId <= 0) return false;
            const record = fullscreen[kind];
            if (Date.now() - record.loadedAt > 55 * 60 * 1000) {
                destroyFullscreen(kind);
                scheduleReload(1000);
                return false;
            }
            active = { id: requestId, kind, slot: record.slot, earned: false };
            const showAd = record.ready;
            record.ready = null;
            try {
                // Some GPT versions return a boolean; older ones return void.
                const result = showAd();
                if (result === false) complete(record.slot, 3);
            } catch (_) { complete(record.slot, 3); }
            return true;
        },
        poll(requestId) {
            const result = results.get(requestId) || 0;
            results.delete(requestId);
            return result;
        },
        setRemoved(value) {
            const next = Boolean(value);
            if (removed === next) return;
            removed = next;
            renderSidebars();
            if (sdkReady && removed && active?.kind !== "interstitial") destroyFullscreen("interstitial");
            if (sdkReady && !removed) loadFullscreen("interstitial");
        },
    });
})();
