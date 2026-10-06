// GraviPop Miniquad Web Plugin
// Handles localStorage persistence, DOM name input alignment, and the shared leaderboard API.
"use strict";

(function () {
    const utf8Encoder = new TextEncoder();

    function writeBytesToWasm(bytes, outPtr, maxLen) {
        if (typeof wasm_memory === "undefined" || !wasm_memory || !outPtr || maxLen <= 0) return 0;
        const len = Math.min(bytes.length, maxLen);
        new Uint8Array(wasm_memory.buffer, outPtr, len).set(bytes.subarray(0, len));
        return len;
    }

    function encodeStringToWasm(str, outPtr, maxLen) {
        const bytes = utf8Encoder.encode(str);
        return writeBytesToWasm(bytes, outPtr, maxLen);
    }

    // Every build uses the same HTTPS API; no browser-only database branch.
    let leaderboardStatus = "";
    let leaderboardDataJson = "[]";
    let isFetching = false;
    let isSubmitting = false;
    let leaderboardVersion = 0;
    function isConfigured() { return true; }
    function leaderboardUrl() { return window.GRAVIPOP_LEADERBOARD_URL || "/api/leaderboard"; }
    async function leaderboardRequest(options) {
        const controller = new AbortController();
        const timeout = setTimeout(() => controller.abort(), 12000);
        try {
            const response = await fetch(leaderboardUrl(), { ...options, signal: controller.signal });
            if (!response.ok) throw new Error("Leaderboard unavailable");
            return await response.json();
        }
        finally { clearTimeout(timeout); }
    }

    function getLocalLeaderboard() {
        try {
            const raw = localStorage.getItem("gravipop.local_leaderboard");
            if (raw) return JSON.parse(raw);
        } catch (_) {}
        return [];
    }

    function saveToLocalLeaderboard(name, score) {
        let entries = getLocalLeaderboard();
        const existingIdx = entries.findIndex(e => e.display_name === name);
        if (existingIdx >= 0) {
            entries[existingIdx].high_score = Math.max(entries[existingIdx].high_score, score);
        } else {
            entries.push({ display_name: name, high_score: score });
        }
        entries.sort((a, b) => b.high_score - a.high_score);
        entries = entries.slice(0, 20);
        try {
            localStorage.setItem("gravipop.local_leaderboard", JSON.stringify(entries));
        } catch (_) {}
        leaderboardDataJson = JSON.stringify(entries);
    }

    async function doLeaderboardRefresh() {
        if (isFetching || isSubmitting) return;
        isFetching = true;
        const version = ++leaderboardVersion;
        leaderboardStatus = "Refreshing global leaderboard...";
        try {
            const data = await leaderboardRequest({ headers: { "Accept": "application/json" } });
            if (!Array.isArray(data)) throw new Error("Invalid leaderboard response");
            if (version !== leaderboardVersion) return;
            leaderboardDataJson = JSON.stringify(data);
            leaderboardStatus = "Global Leaderboard Synchronized";
        } catch (_) {
            if (version !== leaderboardVersion) return;
            leaderboardDataJson = JSON.stringify(getLocalLeaderboard());
            leaderboardStatus = "Offline: showing scores saved on this device";
        } finally { isFetching = false; }
    }

    async function doLeaderboardSubmit(displayName, score) {
        displayName = (displayName || "").trim();
        if (!displayName || displayName.length < 3) {
            leaderboardStatus = "Enter at least 3 characters.";
            return;
        }
        if (isSubmitting) return;
        isSubmitting = true;
        ++leaderboardVersion; // A stale refresh cannot overwrite this submission.
        saveToLocalLeaderboard(displayName, score);
        leaderboardStatus = "Submitting score to global leaderboard...";
        try {
            const data = await leaderboardRequest({
                method: "POST", headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ display_name: displayName, high_score: score })
            });
            if (data.success !== true || !Array.isArray(data.scores)) throw new Error("Invalid submission response");
            leaderboardDataJson = JSON.stringify(data.scores);
            leaderboardStatus = "Score submitted globally!";
        } catch (_) { leaderboardStatus = "Score saved on this device. Cloud sync failed; check your connection."; }
        finally { isSubmitting = false; }
    }

    let submitRequested = false;

    const plugin = {
        name: "gravipop_web",
        version: 1,
        register_plugin: function (importObject) {
            importObject.env = importObject.env || {};

            // Real SDK callbacks are polled by Rust; DOM controls cannot grant rewards.
            importObject.env.gravipop_ads_ready = (rewarded) =>
                window.GravipopAds?.isReady(Boolean(rewarded)) ? 1 : 0;
            importObject.env.gravipop_ads_show = (requestId, rewarded) =>
                window.GravipopAds?.show(requestId >>> 0, Boolean(rewarded)) ? 1 : 0;
            importObject.env.gravipop_ads_poll = (requestId) =>
                window.GravipopAds?.poll(requestId >>> 0) || 0;
            importObject.env.gravipop_ads_set_removed = (removed) =>
                window.GravipopAds?.setRemoved(Boolean(removed));

            // ── Storage FFI ──
            importObject.env.gravipop_storage_len = function (keyPtr, keyLen) {
                try {
                    const key = UTF8ToString(keyPtr, keyLen);
                    const val = localStorage.getItem(key);
                    if (val === null) return -1;
                    return utf8Encoder.encode(val).length;
                } catch (_) {
                    return -1;
                }
            };

            importObject.env.gravipop_storage_get = function (keyPtr, keyLen, outPtr, maxLen) {
                try {
                    const key = UTF8ToString(keyPtr, keyLen);
                    const val = localStorage.getItem(key);
                    if (val === null) return -1;
                    return encodeStringToWasm(val, outPtr, maxLen);
                } catch (_) {
                    return -1;
                }
            };

            importObject.env.gravipop_storage_set = function (keyPtr, keyLen, valPtr, valLen) {
                try {
                    const key = UTF8ToString(keyPtr, keyLen);
                    const val = UTF8ToString(valPtr, valLen);
                    localStorage.setItem(key, val);
                } catch (e) {
                    console.error("gravipop_storage_set failed", e);
                }
            };

            // ── DOM Input FFI ──
            importObject.env.gravipop_sync_name_input = function (show, x, y, w, h, scoreHigh, scoreLow, initPtr, initLen, outPtr, maxLen) {
                const container = document.getElementById("name-input-container");
                const input = document.getElementById("leaderboard-name");
                if (!input) return -1;

                if (!input.dataset.bound) {
                    input.dataset.bound = "1";
                    input.addEventListener("keydown", (e) => {
                        if (e.key === "Enter") {
                            e.preventDefault();
                            submitRequested = true;
                        }
                    });
                }
                const targetEl = container || input;
                if (!show) {
                    submitRequested = false;
                    targetEl.style.display = "none";
                    return -1;
                }

                const canvasEl = document.getElementById("glcanvas");
                const width = canvasEl ? canvasEl.clientWidth : window.innerWidth;
                const height = canvasEl ? canvasEl.clientHeight : window.innerHeight;
                const scale = Math.min(width / 720, height / 1280);
                const ox = (width - 720 * scale) * 0.5;
                const oy = (height - 1280 * scale) * 0.5;

                targetEl.style.display = "block";
                targetEl.style.position = "absolute";
                targetEl.style.left = `${ox + x * scale}px`;
                targetEl.style.top = `${oy + y * scale}px`;
                targetEl.style.width = `${w * scale}px`;
                targetEl.style.height = `${h * scale}px`;

                input.style.fontSize = `${Math.max(12, 18 * scale)}px`;

                if (initLen > 0 && initPtr) {
                    input.value = UTF8ToString(initPtr, initLen);
                }
                return encodeStringToWasm(input.value, outPtr, maxLen);
            };

            // ── Leaderboard FFI ──
            importObject.env.gravipop_take_submit_request = function () {
                const requested = submitRequested;
                submitRequested = false;
                return requested ? 1 : 0;
            };
            importObject.env.gravipop_leaderboard_configured = function () {
                return 1;
            };

            importObject.env.gravipop_leaderboard_refresh = function () {
                doLeaderboardRefresh();
            };

            importObject.env.gravipop_leaderboard_submit = function (namePtr, nameLen, scoreHigh, scoreLow) {
                const name = UTF8ToString(namePtr, nameLen);
                const score = (BigInt(scoreHigh >>> 0) << 32n) | BigInt(scoreLow >>> 0);
                doLeaderboardSubmit(name, Number(score));
            };

            importObject.env.gravipop_leaderboard_status_len = function () {
                if (!leaderboardStatus) return 0;
                return utf8Encoder.encode(leaderboardStatus).length;
            };

            importObject.env.gravipop_leaderboard_status_get = function (outPtr, maxLen) {
                if (!leaderboardStatus) return 0;
                return encodeStringToWasm(leaderboardStatus, outPtr, maxLen);
            };

            importObject.env.gravipop_leaderboard_data_len = function () {
                if (!leaderboardDataJson) return 0;
                return utf8Encoder.encode(leaderboardDataJson).length;
            };

            importObject.env.gravipop_leaderboard_data_get = function (outPtr, maxLen) {
                if (!leaderboardDataJson) return 0;
                return encodeStringToWasm(leaderboardDataJson, outPtr, maxLen);
            };
        }
    };

    if (typeof miniquad_add_plugin !== "undefined") {
        miniquad_add_plugin(plugin);
    } else {
        window.addEventListener("DOMContentLoaded", () => {
            if (typeof miniquad_add_plugin !== "undefined") {
                miniquad_add_plugin(plugin);
            }
        });
    }
})();
