import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";

const source = readFileSync(new URL("../web/gravipop_ads.js", import.meta.url), "utf8");

function harness(options = { mode: "test" }) {
  const listeners = new Map();
  const slots = [];
  const timers = new Map();
  const elements = new Map();
  let nextTimer = 0;
  let width = 1280;
  let height = 800;
  let focusCount = 0;
  const node = () => ({
    hidden: true, textContent: "", style: {},
    querySelector() { return this.label ||= node(); },
    replaceChildren() {},
    getBoundingClientRect() { return { width, height }; },
    focus() { focusCount++; },
  });
  for (const id of ["canvas-wrapper", "ad-rail-left", "ad-rail-right", "ad-rail-top", "ad-rail-bottom", "google-ad-left", "google-ad-right", "google-ad-top", "google-ad-bottom", "ads-status", "glcanvas"]) elements.set(id, node());
  const pubads = { addEventListener(name, callback) { listeners.set(name, callback); } };
  const newSlot = (path, kind, sizes) => {
    const slot = { path, kind, sizes, destroyed: false, addService() { return this; } };
    slots.push(slot);
    return slot;
  };
  const tag = {
    cmd: { push(callback) { callback(); } },
    pubads: () => pubads,
    enums: { OutOfPageFormat: { REWARDED: "rewarded", GAME_MANUAL_INTERSTITIAL: "interstitial" } },
    defineOutOfPageSlot: (path, kind) => newSlot(path, kind),
    defineSlot: (path, sizes, id) => newSlot(path, id, sizes),
    destroySlots(list) { for (const slot of list) slot.destroyed = true; },
    enableServices() {}, display() {},
  };
  const window = { innerWidth: 1920, googletag: tag, addEventListener(name, callback) { listeners.set(`window:${name}`, callback); } };
  const document = { getElementById: (id) => elements.get(id), createElement: node, head: { appendChild() {} } };
  vm.runInNewContext(source, {
    window, document,
    localStorage: { getItem: () => null },
    setTimeout(callback, delay) { const id = ++nextTimer; timers.set(id, { callback, delay }); return id; },
    clearTimeout(id) { timers.delete(id); },
  });
  const api = window.GravipopAds;
  api.configure(options);
  const current = (kind) => slots.findLast((slot) => slot.kind === kind && !slot.destroyed);
  const emit = (name, event) => listeners.get(name)?.(event);
  const ready = (kind, show = () => true) => {
    const slot = current(kind);
    emit(kind === "rewarded" ? "rewardedSlotReady" : "gameManualInterstitialSlotReady", {
      slot, makeRewardedVisible: show, makeGameManualInterstitialVisible: show,
    });
    return slot;
  };
  const flush = (delay) => {
    for (const [id, timer] of [...timers]) {
      if (timer.delay === delay) { timers.delete(id); timer.callback(); }
    }
  };
  return { api, ready, emit, flush, current, elements, slots, window,
    resize(w, h, viewport = 1920) { width = w; height = h; window.innerWidth = viewport; emit("window:resize"); flush(250); },
    focusCount: () => focusCount };
}

{
  const h = harness();
  assert.equal(h.api.show(1, true), false, "an unloaded ad cannot start");
  const slot = h.ready("rewarded");
  assert.equal(h.api.show(1, true), true);
  assert.equal(h.api.show(2, true), false, "double clicks cannot create two requests");
  h.emit("rewardedSlotVideoCompleted", { slot });
  assert.equal(h.api.poll(1), 0, "video completion alone must not grant a reward");
  h.emit("rewardedSlotGranted", { slot: {} });
  h.emit("rewardedSlotClosed", { slot });
  assert.equal(h.api.poll(1), 2, "early closure does not grant a reward");
  assert.equal(h.focusCount(), 1);
  h.flush(1000);
  const next = h.ready("rewarded");
  assert.equal(h.api.show(2, true), true);
  h.emit("rewardedSlotGranted", { slot }); // Old, destroyed ad.
  assert.equal(h.api.poll(2), 0);
  h.emit("rewardedSlotGranted", { slot: next });
  h.emit("rewardedSlotGranted", { slot: next });
  assert.equal(h.api.poll(2), 0, "grant is held until the provider closes the ad");
  h.emit("rewardedSlotClosed", { slot: next });
  h.emit("rewardedSlotClosed", { slot: next });
  assert.equal(h.api.poll(2), 1);
  assert.equal(h.api.poll(2), 0, "rewards are consumed exactly once");
}
{
  const h = harness();
  const slot = h.ready("interstitial");
  assert.equal(h.api.show(3, false), true);
  h.emit("rewardedSlotGranted", { slot });
  h.emit("gameManualInterstitialSlotClosed", { slot });
  assert.equal(h.api.poll(3), 2, "interstitials never grant gameplay rewards");
}
{
  const h = harness();
  h.ready("rewarded", () => false);
  assert.equal(h.api.show(4, true), true);
  assert.equal(h.api.poll(4), 3, "failure to show unblocks Rust without a reward");
  h.flush(1000);
  const slot = h.current("rewarded");
  h.emit("slotRenderEnded", { slot, isEmpty: true });
  assert.equal(h.api.isReady(true), false);
  h.flush(30000);
  assert.notEqual(h.current("rewarded"), slot, "no-fill is retried with a fresh slot");
}
{
  const h = harness();
  assert.equal(h.elements.get("ad-rail-left").hidden, false);
  for (const slot of h.slots.filter((slot) => slot.sizes)) {
    for (const [w, sizeHeight] of slot.sizes) {
      assert.ok(w <= (1280 - 450) / 2 - 32 && sizeHeight <= 728, "sidebar creatives fit outside the playable canvas");
    }
  }
  h.resize(600, 800, 600);
  assert.equal(h.elements.get("ad-rail-left").hidden, true);
  h.resize(1280, 800);
  assert.equal(h.elements.get("ad-rail-right").hidden, false);
  h.api.setRemoved(true);
  assert.equal(h.elements.get("ad-rail-left").hidden, true);
  assert.equal(h.current("interstitial"), undefined);
  h.ready("rewarded");
  assert.equal(h.api.isReady(true), true, "ad removal keeps player-selected rewarded ads available");
}
{
  const live = harness({ mode: "live", left: "/123456/MyGame/Left" });
  assert.equal(live.current("rewarded"), undefined);
  assert.equal(live.current("interstitial"), undefined);
  assert.ok(live.slots.every((slot) => slot.path.startsWith("/123456/")), "missing live IDs never use test inventory");
  const off = harness({ mode: "off" });
  assert.equal(off.slots.length, 0);
}
console.log("Real-ad lifecycle checks passed: exact-once rewards, cancellation, no-fill, interstitials, responsive sidebar placement, ad removal, and live configuration.");
