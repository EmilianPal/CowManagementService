const { test } = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { execFileSync } = require("node:child_process");
const { syncVersions } = require("../scripts/sync-version.cjs");
const root = path.resolve(__dirname, "..");
const files = ["package.json", "package-lock.json", "src-tauri/Cargo.toml", "src-tauri/Cargo.lock", "src-tauri/tauri.conf.json", "scripts/sync-version.cjs", "scripts/version.cjs"];

function fixture(t) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "ferma-version-"));
  for (const file of files) {
    fs.mkdirSync(path.dirname(path.join(dir, file)), { recursive: true });
    fs.copyFileSync(path.join(root, file), path.join(dir, file));
  }
  t.after(() => {
    assert.equal(path.dirname(path.resolve(dir)), path.resolve(os.tmpdir()));
    fs.rmSync(dir, { recursive: true, force: true });
  });
  return dir;
}
const read = (dir, file) => fs.readFileSync(path.join(dir, file), "utf8");
const json = (dir, file) => JSON.parse(read(dir, file));
function setSource(dir, version) {
  fs.writeFileSync(path.join(dir, "package.json"), JSON.stringify({ ...json(dir, "package.json"), version }, null, 2) + "\n");
}
function assertVersions(dir, version) {
  assert.equal(json(dir, "package.json").version, version);
  assert.equal(json(dir, "package-lock.json").version, version);
  assert.equal(json(dir, "package-lock.json").packages[""].version, version);
  assert.equal(json(dir, "src-tauri/tauri.conf.json").version, version);
  assert.ok(read(dir, "src-tauri/Cargo.toml").includes(`version = "${version}"`));
  assert.ok(read(dir, "src-tauri/Cargo.lock").includes(`name = "cowmanagementservice"\nversion = "${version}"`) ||
    read(dir, "src-tauri/Cargo.lock").includes(`name = "cowmanagementservice"\r\nversion = "${version}"`));
}

test("One source synchronizes all versions without changing dependencies or app identity", t => {
  const dir = fixture(t);
  const originalLock = json(dir, "package-lock.json");
  setSource(dir, "1.2.3");
  syncVersions(dir);
  assertVersions(dir, "1.2.3");
  assert.equal(json(dir, "src-tauri/tauri.conf.json").identifier, "com.lenovo.cowmanagementservice");
  const lock = json(dir, "package-lock.json");
  for (const key of Object.keys(originalLock.packages).filter(Boolean)) {
    assert.deepEqual(lock.packages[key], originalLock.packages[key]);
  }
  const cargoDependencies = source => source.split(/(?=^\[\[package\]\])/m).filter(block => !block.includes('name = "cowmanagementservice"')).join("");
  assert.equal(cargoDependencies(read(dir, "src-tauri/Cargo.lock")), cargoDependencies(read(root, "src-tauri/Cargo.lock")));
  const snapshot = files.map(file => read(dir, file));
  syncVersions(dir);
  assert.deepEqual(files.map(file => read(dir, file)), snapshot);
});

test("Malformed inputs fail before writing any target files", t => {
  const dir = fixture(t);
  setSource(dir, "invalid");
  const snapshot = files.map(file => read(dir, file));
  assert.throws(() => syncVersions(dir), /valid app version/);
  assert.deepEqual(files.map(file => read(dir, file)), snapshot);
  setSource(dir, "1.2.3");
  fs.writeFileSync(path.join(dir, "src-tauri/Cargo.lock"), "not a package lock");
  const broken = files.map(file => read(dir, file));
  assert.throws(() => syncVersions(dir), /Cannot identify/);
  assert.deepEqual(files.map(file => read(dir, file)), broken);
});

test("The npm command runs the synchronization hook for exact and patch versions", t => {
  const dir = fixture(t);
  const npmCli = process.env.npm_execpath || path.join(path.dirname(process.execPath), "node_modules/npm/bin/npm-cli.js");
  if (!fs.existsSync(npmCli)) return t.skip("npm CLI is unavailable in this environment");
  const run = version => execFileSync(process.execPath,
    [npmCli, "run", "version:set", "--", version],
    { cwd: dir, encoding: "utf8", env: { ...process.env, npm_config_ignore_scripts: "false" } });
  run("9.8.7");
  assertVersions(dir, "9.8.7");
  run("patch");
  assertVersions(dir, "9.8.8");
});

test("The version commands read the source and bump big, medium, and small", t => {
  const dir = fixture(t);
  const command = (...args) => execFileSync(process.execPath, [path.join(dir, "scripts/version.cjs"), ...args],
    { cwd: dir, encoding: "utf8" }).trim();
  assert.equal(command("get"), `Current version: ${json(dir, "package.json").version}`);
  for (const [from, size, to] of [
    ["1.4.3", "big", "2.0.0"],
    ["1.5.2", "medium", "1.6.0"],
    ["1.7.3", "small", "1.7.4"],
  ]) {
    setSource(dir, from);
    assert.equal(command("get"), `Current version: ${from}`);
    assert.equal(command("bump", size), `Version changed from ${from} to ${to}.`);
    assertVersions(dir, to);
  }
});

test("Invalid bump arguments and failed synchronization leave versions unchanged", t => {
  const dir = fixture(t);
  const { bumpVersion, nextVersion } = require("../scripts/version.cjs");
  assert.throws(() => nextVersion("1.0.1", "huge"), /big, medium, or small/);
  assert.throws(() => nextVersion("1.0.1-beta", "small"), /three numeric parts/);
  const snapshot = files.map(file => read(dir, file));
  assert.throws(() => bumpVersion(dir, undefined), /big, medium, or small/);
  assert.deepEqual(files.map(file => read(dir, file)), snapshot);
  fs.writeFileSync(path.join(dir, "src-tauri/Cargo.lock"), "invalid lock");
  const broken = files.map(file => read(dir, file));
  assert.throws(() => bumpVersion(dir, "small"), /Cannot identify/);
  assert.deepEqual(files.map(file => read(dir, file)), broken);
});
