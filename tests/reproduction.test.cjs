const { test } = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const ts = require("typescript");
function load(file, requireFn = require) {
  const code = ts.transpileModule(fs.readFileSync(path.join(__dirname, "../src/features/herd", file), "utf8"), { compilerOptions: { module: ts.ModuleKind.CommonJS } }).outputText;
  const exports = {};
  vm.runInNewContext(code, { exports, require: requireFn });
  return exports;
}
const types = load("types.ts");
const { availableCalves, calfAssociationError } = load("reproduction.ts", () => types);
const mother = { id: 1, ear_tag: "RO1234567", birth_date: "2019-07-04", birth_id: null };
const calf = { id: 2, ear_tag: "RO500010699567", birth_date: "2025-02-03", birth_id: null };
const assigned = { ...calf, id: 3, birth_id: 10 };
test("Mother is excluded and other unassigned cattle remain selectable", () => {
  assert.deepEqual(Array.from(availableCalves([mother, calf, assigned], 1), c => c.id), [2]);
  assert.deepEqual(Array.from(availableCalves([mother, calf, assigned], 2), c => c.id), [1]);
});
test("A distinct calf with its birth date is a valid association", () => {
  assert.equal(calfAssociationError(calf, mother.id, calf.birth_date), "");
});
test("Date mismatch identifies the calf and both dates without claiming it is the mother", () => {
  const error = calfAssociationError(calf, mother.id, "2026-10-29");
  assert.match(error, /RO500010699567/);
  assert.match(error, /03\.02\.2025/);
  assert.match(error, /29\.10\.2026/);
  assert.doesNotMatch(error, /mamă/);
});
test("Self-parenting and existing associations have separate errors", () => {
  assert.match(calfAssociationError(mother, mother.id, mother.birth_date), /Mama/);
  assert.match(calfAssociationError(assigned, mother.id, assigned.birth_date), /deja asociat/);
});

test("The current calf is selectable and valid while calves from other births stay excluded", () => {
  const another = { ...assigned, id: 4, birth_id: 20 };
  assert.deepEqual(Array.from(availableCalves([mother, calf, assigned, another], mother.id, assigned.id), c => c.id), [2, 3]);
  assert.equal(calfAssociationError(assigned, mother.id, assigned.birth_date, 10), "");
  assert.match(calfAssociationError(another, mother.id, another.birth_date, 10), /deja asociat/);
  assert.deepEqual(Array.from(availableCalves([mother, calf], mother.id, mother.id), c => c.id), [2]);
});
