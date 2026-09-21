const { test } = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const vm = require("node:vm");
const ts = require("typescript");

const source = fs.readFileSync(require("node:path").join(__dirname, "../src/lib/api.ts"), "utf8");
const code = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 } }).outputText;

for (const [original, expected] of [
  ["Invalid username or password", "Numele de utilizator sau parola este incorectă."],
  ["Failed to create farm: UNIQUE constraint failed: farms.name", "Există deja o fermă cu acest nume. Autentifică-te sau alege alt nume."],
  ["Failed to create user: UNIQUE constraint failed: users.email", "Această adresă de e-mail este deja folosită. Autentifică-te sau folosește altă adresă."],
  ["Failed to create farm: no such table: farm", "Baza de date nu are structura necesară. Repornește aplicația după actualizare."],
]) {
  test(`Mesajul tradus se păstrează: ${original}`, async () => {
    const exports = {};
    vm.runInNewContext(code, { exports, require: () => ({ isTauri: () => true, invoke: async () => { throw original; } }) });
    try {
      await exports.command("register_admin");
      assert.fail("Comanda trebuia să eșueze.");
    } catch (error) {
      assert.equal(exports.romanianError(error), expected);
    }
  });
}
