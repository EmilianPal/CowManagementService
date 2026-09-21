const { test } = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const vm = require("node:vm");
const ts = require("typescript");
const source = fs.readFileSync(
  require("node:path").join(__dirname, "../src/features/herd/filters.ts"),
  "utf8",
);
const code = ts.transpileModule(source, {
  compilerOptions: {
    module: ts.ModuleKind.CommonJS,
    target: ts.ScriptTarget.ES2020,
  },
}).outputText;
const api = {};
vm.runInNewContext(code, { exports: api });

test("Toate criteriile ajung în interogarea comună pentru registru și export", () => {
  const result = api.toCowFilter(
    {
      ...api.emptyFilter,
      born_in_year: "2024",
      minimum_age_months: "0",
      maximum_age_months: "12",
      entered_on: "2024-02-01",
      exited_on: "2025-02-01",
      breed: "Metis",
      sex: "Female",
      category: "Carne",
    },
    " ro123 ",
    "2025-02-01",
    "exited",
  );
  assert.deepEqual(JSON.parse(JSON.stringify(result)), {
    date: "2025-02-01",
    ear_tag_contains: "ro123",
    breed: "Metis",
    sex: "Female",
    category: "Carne",
    born_in_year: 2024,
    minimum_age_months: 0,
    maximum_age_months: 12,
    entered_on: "2024-02-01",
    exited_on: "2025-02-01",
    show_only_entered: false,
    show_only_exited: true,
  });
});
test("Limitele invalide sunt respinse, iar câmpurile goale nu restrâng rezultatele", () => {
  assert.ok(
    api.validateFilters({ ...api.emptyFilter, minimum_age_months: "-1" }),
  );
  assert.ok(
    api.validateFilters({ ...api.emptyFilter, maximum_age_months: "1.5" }),
  );
  assert.ok(
    api.validateFilters({
      ...api.emptyFilter,
      minimum_age_months: "12",
      maximum_age_months: "2",
    }),
  );
  assert.equal(api.validateFilters(api.emptyFilter), "");
  assert.ok(api.validateFilters({ ...api.emptyFilter, minimum_age_months: "12", maximum_age_months: "12" }));
  const result = api.toCowFilter(api.emptyFilter, "", "2025-01-01", "all");
  assert.equal(result.minimum_age_months, null);
  assert.equal(result.ear_tag_contains, null);
  assert.equal(result.show_only_entered, false);
});
