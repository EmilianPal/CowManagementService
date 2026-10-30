const { test } = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const ts = require("typescript");
const source = fs.readFileSync(path.join(__dirname, "../src/features/herd/eventFilters.ts"), "utf8");
const code = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.CommonJS } }).outputText;
const api = {};
vm.runInNewContext(code, { exports: api });
const { birthRows, inseminationRows, emptyEventFilters } = api;
const filter = values => ({ ...emptyEventFilters, ...values });
const cows = [
  { id: 1, ear_tag: "RO-MOTHER-67", birth_id: null, breed: "BaltataRomaneasca", sex: "Female" },
  { id: 2, ear_tag: "RO-MOTHER-80", birth_id: null, breed: "Metis", sex: "Female" },
  { id: 3, ear_tag: "RO-SIRE-67", birth_id: null, breed: "Metis", sex: "Male" },
  { id: 4, ear_tag: "RO-CALF-01", birth_id: 10, breed: "BaltataRomaneasca", sex: "Female", birth_date: "2026-01-07" },
  { id: 5, ear_tag: "RO-CALF-02", birth_id: 10, breed: "Metis", sex: "Male", birth_date: "2026-01-07" },
  { id: 6, ear_tag: "RO-CALF-67", birth_id: 11, breed: "BaltataRomaneasca", sex: "Female", birth_date: "2025-01-07" },
];
const births = [
  { id: 10, mother_id: 1, date: "2026-01-07" },
  { id: 11, mother_id: 2, date: "2025-01-07" },
  { id: 12, mother_id: 1, date: "2026-02-01" },
  { id: 13, mother_id: 1, date: "2026-01-31" },
];
const events = [
  { id: 20, dam_id: 1, sire_id: 3, date: "2026-01-01" },
  { id: 21, dam_id: 2, sire_id: null, date: "2026-01-31" },
  { id: 22, dam_id: 1, sire_id: 3, date: "2025-01-31" },
  { id: 23, dam_id: 1, sire_id: null, date: "2026-02-01" },
];
test("January in a specific year includes the whole month and excludes other years and months", () => {
  assert.deepEqual(Array.from(birthRows(births, cows, filter({ year: "2026", month: "01" })), r => r.birth.id), [13, 10, 10]);
  assert.deepEqual(Array.from(inseminationRows(events, cows, filter({ year: "2026", month: "01" })), r => r.event.id), [21, 20]);
});
test("Month alone spans years; year alone includes every month", () => {
  assert.equal(birthRows(births, cows, filter({ month: "01" })).length, 4);
  assert.equal(inseminationRows(events, cows, filter({ year: "2026" })).length, 3);
});
test("Twins have separate identification rows and the correct shared mother", () => {
  const rows = birthRows(births, cows, filter({ year: "2026", month: "01", association: "with" }));
  assert.equal(rows.length, 2);
  assert.equal(new Set(rows.map(r => r.birth.id)).size, 1);
  assert.equal(rows[0].calf.ear_tag, "RO-CALF-01");
  assert.equal(rows[0].calf.sex, "Female");
  assert.equal(rows[0].calf.breed, "BaltataRomaneasca");
  assert.equal(rows[0].calf.birth_date, "2026-01-07");
  assert.ok(rows.every(r => r.mother.id === 1));
});
test("All calf filters apply to the same row, without including a nonmatching twin", () => {
  const f = filter({ year: "2026", month: "01", mother: " 67 ", related: "calf-01", breed: "BaltataRomaneasca", sex: "Female" });
  assert.deepEqual(Array.from(birthRows(births, cows, f), r => r.calf.id), [4]);
  assert.equal(birthRows(births, cows, { ...f, sex: "Male" }).length, 0);
});
test("Unregistered calves remain visible as births; calf-specific filters exclude missing data", () => {
  const rows = birthRows(births, cows, filter({ association: "without" }));
  assert.deepEqual(Array.from(rows, r => r.birth.id), [12, 13]);
  assert.ok(rows.every(r => !r.calf));
  assert.equal(birthRows(births, cows, filter({ association: "without", sex: "Female" })).length, 0);
});
test("Inseminations support both parents, breed, and an unspecified sire", () => {
  assert.deepEqual(Array.from(inseminationRows(events, cows, filter({ year: "2026", mother: "67", related: "sire-67", breed: "BaltataRomaneasca" })), r => r.event.id), [20]);
  assert.deepEqual(Array.from(inseminationRows(events, cows, filter({ association: "without" })), r => r.event.id), [23, 21]);
});
test("Cleared filters restore all records, empty sets are handled, and input data is unchanged", () => {
  const snapshot = JSON.stringify({ births, cows, events });
  assert.equal(birthRows(births, cows, filter({})).length, 5);
  assert.equal(inseminationRows(events, cows, filter({})).length, 4);
  assert.equal(birthRows([], cows, filter({})).length, 0);
  assert.equal(inseminationRows([], cows, filter({})).length, 0);
  assert.equal(JSON.stringify({ births, cows, events }), snapshot);
});
