import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

const element = () => ({ dataset: {}, style: {}, handlers: {},
  addEventListener(event, handler) { this.handlers[event] = handler; } });
const input = { ...element(), value: 'Commander' };
const container = element();
let plugin;
let requests = 0;
vm.runInNewContext(readFileSync(new URL('../web/gravipop_web.js', import.meta.url), 'utf8'), {
  TextEncoder, window: {},
  document: { getElementById(id) { return ({ 'leaderboard-name': input,
    'name-input-container': container,
    glcanvas: { clientWidth: 720, clientHeight: 1280 } })[id]; } },
  fetch() { requests++; throw new Error('DOM events must not write scores directly'); },
  miniquad_add_plugin(value) { plugin = value; },
});
const imports = { env: {} };
plugin.register_plugin(imports);
const sync = (show) => imports.env.gravipop_sync_name_input(show, 0, 0, 300, 50, 0, 100, 0, 0, 0, 0);
sync(1);
assert.equal(requests, 0);
assert.equal(imports.env.gravipop_take_submit_request(), 0);
input.handlers.keydown({ key: 'Enter', preventDefault() {} });
assert.equal(imports.env.gravipop_take_submit_request(), 1);
sync(0);
assert.equal(imports.env.gravipop_take_submit_request(), 0);
const html = readFileSync(new URL('../index.html', import.meta.url), 'utf8');
assert.doesNotMatch(html, /leaderboard-submit-btn/);
console.log('Callsign input has no duplicate submit control; Enter queues one canvas save action.');
