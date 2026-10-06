// GraviPop Miniquad Web Plugin
// Handles localStorage persistence, DOM name input alignment, and Supabase Leaderboard REST calls.
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

    // Leaderboard state
    const SUPABASE_URL = (window.GRAVIPOP_SUPABASE_URL || "").replace(/\/+$/, "");
    const SUPABASE_KEY = window.GRAVIPOP_SUPABASE_ANON_KEY || "";
    let leaderboardStatus = "";
    let leaderboardDataJson = "[]";
    let isFetching = false;

    function isConfigured() {
        return Boolean(SUPABASE_URL && SUPABASE_KEY);
    }

    async function ensureSession() {
        let session = null;
        try {
            const stored = localStorage.getItem("gravipop.auth.session");
            if (stored) session = JSON.parse(stored);
        } catch (_) {}

        const now = Date.now();
        if (session && session.expires_at_ms && session.expires_at_ms > now + 30000) {
            return session;
        }

        if (session && session.refresh_token) {
            try {
                const res = await fetch(`${SUPABASE_URL}/auth/v1/token?grant_type=refresh_token`, {
                    method: "POST",
                    headers: {
                        "apikey": SUPABASE_KEY,
                        "Content-Type": "application/json"
                    },
                    body: JSON.stringify({ refresh_token: session.refresh_token })
                });
                if (res.ok) {
                    const refreshed = await res.json();
                    refreshed.expires_at_ms = Date.now() + (refreshed.expires_in || 3600) * 1000;
                    localStorage.setItem("gravipop.auth.session", JSON.stringify(refreshed));
                    return refreshed;
                }
            } catch (_) {}
        }

        const signupRes = await fetch(`${SUPABASE_URL}/auth/v1/signup`, {
            method: "POST",
            headers: {
                "apikey": SUPABASE_KEY,
                "Content-Type": "application/json"
            },
            body: JSON.stringify({ data: {} })
        });
        if (!signupRes.ok) {
            const text = await signupRes.text();
            throw new Error(`Auth failed (${signupRes.status}): ${text}`);
        }
        const newSession = await signupRes.json();
        newSession.expires_at_ms = Date.now() + (newSession.expires_in || 3600) * 1000;
        localStorage.setItem("gravipop.auth.session", JSON.stringify(newSession));
        return newSession;
    }

    async function doLeaderboardRefresh() {
        if (!isConfigured()) {
            leaderboardStatus = "Leaderboard is not configured yet";
            return;
        }
        if (isFetching) return;
        isFetching = true;
        leaderboardStatus = "Refreshing leaderboard...";
        try {
            const session = await ensureSession();
            const token = session?.access_token || SUPABASE_KEY;
            const res = await fetch(`${SUPABASE_URL}/rest/v1/rpc/get_gravipop_leaderboard`, {
                method: "POST",
                headers: {
                    "apikey": SUPABASE_KEY,
                    "Authorization": `Bearer ${token}`,
                    "Content-Type": "application/json",
                    "Accept": "application/json"
                },
                body: JSON.stringify({ p_limit: 20 })
            });
            if (!res.ok) {
                const err = await res.text();
                leaderboardStatus = `Error ${res.status}: ${err}`;
                return;
            }
            const data = await res.json();
            leaderboardDataJson = JSON.stringify(data);
            leaderboardStatus = "";
        } catch (err) {
            leaderboardStatus = `Network error: ${err.message || err}`;
        } finally {
            isFetching = false;
        }
    }

    async function doLeaderboardSubmit(displayName, score) {
        if (!isConfigured()) return;
        leaderboardStatus = "Submitting score...";
        try {
            const session = await ensureSession();
            const token = session?.access_token || SUPABASE_KEY;
            const res = await fetch(`${SUPABASE_URL}/rest/v1/rpc/submit_gravipop_score`, {
                method: "POST",
                headers: {
                    "apikey": SUPABASE_KEY,
                    "Authorization": `Bearer ${token}`,
                    "Content-Type": "application/json"
                },
                body: JSON.stringify({ p_display_name: displayName, p_score: score })
            });
            if (!res.ok) {
                const err = await res.text();
                leaderboardStatus = `Submit error ${res.status}: ${err}`;
                return;
            }
            await doLeaderboardRefresh();
        } catch (err) {
            leaderboardStatus = `Submit error: ${err.message || err}`;
        }
    }

    const plugin = {
        name: "gravipop_web",
        version: 1,
        register_plugin: function (importObject) {
            importObject.env = importObject.env || {};

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
            importObject.env.gravipop_sync_name_input = function (show, x, y, w, h, initPtr, initLen, outPtr, maxLen) {
                const input = document.getElementById("leaderboard-name");
                if (!input) return -1;
                if (!show) {
                    input.style.display = "none";
                    return -1;
                }
                const canvasEl = document.getElementById("glcanvas");
                const rect = canvasEl ? canvasEl.getBoundingClientRect() : { left: 0, top: 0, width: window.innerWidth, height: window.innerHeight };
                const width = rect.width;
                const height = rect.height;
                const scale = Math.min(width / 720, height / 1280);
                const ox = (width - 720 * scale) * 0.5;
                const oy = (height - 1280 * scale) * 0.5;

                input.style.display = "block";
                input.style.left = `${rect.left + ox + x * scale}px`;
                input.style.top = `${rect.top + oy + y * scale}px`;
                input.style.width = `${w * scale}px`;
                input.style.height = `${h * scale}px`;
                input.style.fontSize = `${Math.max(12, 19 * scale)}px`;

                if (initLen > 0 && initPtr) {
                    input.value = UTF8ToString(initPtr, initLen);
                }
                return encodeStringToWasm(input.value, outPtr, maxLen);
            };

            // ── Leaderboard FFI ──
            importObject.env.gravipop_leaderboard_configured = function () {
                return isConfigured() ? 1 : 0;
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
