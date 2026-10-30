const { test } = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const ts = require("typescript");

const source = fs.readFileSync(path.join(__dirname, "../src/components/CalfMultiSelect.tsx"), "utf8");
const code = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.CommonJS, jsx: ts.JsxEmit.ReactJSX },
}).outputText;
const api = {};
vm.runInNewContext(code, {
  exports: api,
  require: name => name.includes("herd/types") ? { breeds: {} } : require(name),
});

const cows = [
  { id: 1, ear_tag: "RO1234567", birth_date: "2026-10-21" },
  { id: 2, ear_tag: "RO6700123", birth_date: "2026-10-21" },
  { id: 3, ear_tag: "RO1234500", birth_date: "2026-10-21" },
  { id: 4, ear_tag: "RO9876700", birth_date: "2024-01-07" },
];
const ids = (selected, query, date = "2026-10-21") =>
  Array.from(api.matchingCalves(cows, selected, query, date), cow => cow.id);

test("Selected calves appear first without changing the order within each group", () => {
  assert.deepEqual(ids([3, 1], ""), [1, 3, 2]);
});

test("Ear-tag search matches any fragment and keeps selected matches first", () => {
  assert.deepEqual(ids([2], " 67 "), [2, 1]);
  assert.deepEqual(ids([3], "ro123"), [3, 1]);
  assert.deepEqual(ids([1], "999"), []);
});

test("Only calves born on the event date appear, even if another date's calf was selected", () => {
  assert.deepEqual(ids([4], ""), [1, 2, 3]);
  assert.deepEqual(ids([1, 4], "67", "2024-01-07"), [4]);
});
