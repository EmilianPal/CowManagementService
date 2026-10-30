const { test } = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const vm = require("node:vm");
const ts = require("typescript");
const source = fs.readFileSync(require("node:path").join(__dirname, "../src/components/CowSelect.tsx"), "utf8");
const code = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.CommonJS, jsx: ts.JsxEmit.ReactJSX } }).outputText;
const api = {};
vm.runInNewContext(code, { exports: api, require: name => name.includes("herd/types") ? { breeds: {} } : require(name) });
const cows = [{ id: 1, ear_tag: "RO1234567" }, { id: 2, ear_tag: "RO6700123" }, { id: 3, ear_tag: "RO1234500" }];
test("Search matches a fragment anywhere in the ear tag and excludes other cows", () => {
  assert.deepEqual(Array.from(api.matchingCows(cows, "67"), c => c.id), [1, 2]);
});
test("Search ignores case and surrounding whitespace; empty and missing matches work", () => {
  assert.deepEqual(Array.from(api.matchingCows(cows, " ro123 "), c => c.id), [1, 3]);
  assert.equal(api.matchingCows(cows, "").length, 3);
  assert.equal(api.matchingCows(cows, "999").length, 0);
});
