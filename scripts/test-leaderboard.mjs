import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import handler from "../api/leaderboard.js";

function webClient(fetch) {
  const memory = { buffer: new ArrayBuffer(65536) };
  const records = new Map();
  let plugin;
  vm.runInNewContext(readFileSync(new URL("../web/gravipop_web.js", import.meta.url), "utf8"), {
    window: {}, document: {}, TextEncoder, fetch, AbortController, setTimeout, clearTimeout,
    wasm_memory: memory,
    UTF8ToString(pointer, length) { return new TextDecoder().decode(new Uint8Array(memory.buffer, pointer, length)); },
    localStorage: { getItem: (key) => records.get(key) || null, setItem: (key, value) => records.set(key, value) },
    miniquad_add_plugin(value) { plugin = value; },
  });
  const imports = { env: {} };
  plugin.register_plugin(imports);
  const api = imports.env;
  const decode = (kind) => {
    const length = api[`gravipop_leaderboard_${kind}_len`]();
    api[`gravipop_leaderboard_${kind}_get`](4096, length);
    return new TextDecoder().decode(new Uint8Array(memory.buffer, 4096, length));
  };
  return {
    api, records, data: () => JSON.parse(decode("data")), status: () => decode("status"),
    submit(name, score) {
      const bytes = new TextEncoder().encode(name);
      new Uint8Array(memory.buffer, 1024, bytes.length).set(bytes);
      api.gravipop_leaderboard_submit(1024, bytes.length, 0, score);
    },
  };
}
const settle = () => new Promise((resolve) => setImmediate(resolve));
const response = (data, ok = true) => ({ ok, json: async () => data });

{
  const requests = [];
  const web = webClient(async (url, options) => {
    requests.push({ url, options });
    return response(options.method === "POST"
      ? { success: true, scores: [{ display_name: "Android Pilot", high_score: 1234 }] }
      : [{ display_name: "Desktop Pilot", high_score: 555 }]);
  });
  web.api.gravipop_leaderboard_refresh();
  await settle();
  assert.equal(web.data()[0].display_name, "Desktop Pilot");
  web.submit("Android Pilot", 1234);
  web.submit("Android Pilot", 1234); // In-flight duplicate cannot write twice.
  await settle();
  assert.equal(requests.length, 2);
  assert.ok(requests.every(({ url }) => url === "/api/leaderboard"));
  assert.deepEqual(JSON.parse(requests[1].options.body), { display_name: "Android Pilot", high_score: 1234 });
  assert.equal(web.data()[0].high_score, 1234);
  assert.match(web.status(), /submitted globally/);
}
{
  const web = webClient(async () => response([]));
  web.api.gravipop_leaderboard_refresh();
  await settle();
  assert.deepEqual(web.data(), []); // Empty global board cannot become fabricated scores.
}
{
  const web = webClient(async () => { throw new Error("offline"); });
  web.submit("Offline Pilot", 900);
  await settle();
  web.api.gravipop_leaderboard_refresh();
  await settle();
  assert.equal(web.data()[0].display_name, "Offline Pilot");
  assert.match(web.status(), /Offline/);
}
{
  let resolveRefresh;
  const web = webClient(async (_, options) => options.method === "POST"
    ? response({ success: true, scores: [{ display_name: "New Pilot", high_score: 2000 }] })
    : new Promise((resolve) => { resolveRefresh = resolve; }));
  web.api.gravipop_leaderboard_refresh();
  web.submit("New Pilot", 2000);
  await settle();
  resolveRefresh(response([{ display_name: "Stale Pilot", high_score: 10 }]));
  await settle();
  assert.equal(web.data()[0].display_name, "New Pilot");
}

// Verify DELETE method validation
{
  const res = { headers: {}, setHeader(key, value) { this.headers[key] = value; }, end(body) { this.body = body; } };
  await handler({ method: "DELETE", url: "/api/leaderboard" }, res);
  assert.equal(res.statusCode, 400);
  assert.ok(JSON.parse(res.body).error);
}

// Verify that a provider outage is explicitly offline for every build.
const originalFetch = globalThis.fetch;
const originalError = console.error;
try {
  globalThis.fetch = async () => { throw new Error("provider offline"); };
  console.error = () => {};
  const res = { headers: {}, setHeader(key, value) { this.headers[key] = value; }, end(body) { this.body = body; } };
  await handler({ method: "GET" }, res);
  assert.equal(res.statusCode, 503);
  assert.equal(res.headers["Cache-Control"], "no-store");
  assert.ok(JSON.parse(res.body).error);
} finally { globalThis.fetch = originalFetch; console.error = originalError; }

console.log("Shared leaderboard checks passed: web/native API contract, single submission, stale-response ordering, empty boards, DELETE endpoint, and explicit offline state.");
