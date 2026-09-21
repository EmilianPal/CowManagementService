const { test } = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const vm = require("node:vm");
const ts = require("typescript");
const source = fs.readFileSync(
  require("node:path").join(__dirname, "../src/features/herd/extract.ts"),
  "utf8",
);
const api = {};
vm.runInNewContext(
  ts.transpileModule(source, {
    compilerOptions: {
      module: ts.ModuleKind.CommonJS,
      target: ts.ScriptTarget.ES2020,
    },
  }).outputText,
  { exports: api },
);
let id = 0;
const cow = (birth_date, sex = "Male", changes = {}) => ({
  id: ++id,
  farm_id: 42,
  ear_tag: `RO${String(id).padStart(12, "0")}`,
  birth_date,
  entry_date: birth_date,
  exit_date: null,
  sex,
  breed: "Metis",
  category: "Carne",
  birth_id: null,
  birth_count: 0,
  insemination_count: 0,
  ...changes,
});
const counts = (result) =>
  result.groups.map((g) => [
    g.males.length,
    g.females.length,
    g.males.length + g.females.length,
  ]);
const plain = (value) => JSON.parse(JSON.stringify(value));

test("Extrasul însumează cele șase grupe și reproduce exemplul cu 80 de bovine", () => {
  const cows = [];
  for (const [birth, males, females] of [
    ["2020-01-01", 7, 38],
    ["2025-01-01", 10, 12],
    ["2026-07-01", 12, 1],
  ]) {
    for (let i = 0; i < males; i++) cows.push(cow(birth));
    for (let i = 0; i < females; i++) cows.push(cow(birth, "Female"));
  }
  const result = api.buildExtract(cows.reverse(), "2026-09-21");
  assert.deepEqual(plain(counts(result)), [
    [7, 38, 45],
    [10, 12, 22],
    [12, 1, 13],
  ]);
  assert.equal(result.males, 29);
  assert.equal(result.females, 51);
  assert.equal(result.total, 80);
  const members = result.groups.flatMap((g) => [...g.males, ...g.females]);
  assert.equal(new Set(members.map((c) => c.id)).size, 80);
  for (const group of result.groups) {
    for (const list of [group.males, group.females]) {
      const tags = plain(list.map((c) => c.ear_tag));
      assert.deepEqual(tags, [...tags].sort());
    }
  }
});

test("Pragurile de 6 luni și 2 ani nu omit și nu dublează bovine", () => {
  const result = api.buildExtract(
    [
      cow("2024-09-20"),
      cow("2024-09-21"),
      cow("2024-09-22"),
      cow("2026-03-20"),
      cow("2026-03-21"),
      cow("2026-03-22"),
    ],
    "2026-09-21",
  );
  assert.deepEqual(plain(counts(result)), [
    [1, 0, 1],
    [4, 0, 4],
    [1, 0, 1],
  ]);
});

test("Data extrasului exclude ieșirile, intrările viitoare și bovinele nenăscute", () => {
  const cows = [
    cow("2024-01-01", "Male", { entry_date: "2026-09-21" }),
    cow("2024-01-01", "Male", { entry_date: "2026-09-22" }),
    cow("2024-01-01", "Female", { exit_date: "2026-09-21" }),
    cow("2024-01-01", "Female", { exit_date: "2026-09-22" }),
    cow("2026-09-22", "Female", { entry_date: "2024-01-01" }),
  ];
  const result = api.buildExtract(cows, "2026-09-21");
  assert.equal(result.total, 2);
  assert.equal(result.males, 1);
  assert.equal(result.females, 1);
  assert.equal(api.buildExtract(cows, "2026-09-20").females, 2);
});

test("Aniversările tratează lunile scurte și anii bisecți", () => {
  const leapCow = cow("2024-02-29");
  assert.equal(
    api.buildExtract([leapCow], "2026-02-28").groups[1].males.length,
    1,
  );
  assert.equal(
    api.buildExtract([leapCow], "2026-03-01").groups[0].males.length,
    1,
  );
  const augustCow = cow("2025-08-31");
  assert.equal(
    api.buildExtract([augustCow], "2026-02-27").groups[2].males.length,
    1,
  );
  assert.equal(
    api.buildExtract([augustCow], "2026-02-28").groups[1].males.length,
    1,
  );
});

test("Extrasul gol are total zero, iar data invalidă este respinsă", () => {
  assert.equal(api.buildExtract([], "2026-09-21").total, 0);
  for (const date of ["", "2026-02-30", "invalid", "0000-01-01"]) {
    assert.equal(api.isExtractDate(date), false);
    assert.throws(() => api.buildExtract([], date));
  }
});
