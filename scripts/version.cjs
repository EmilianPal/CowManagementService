const fs = require("node:fs");
const path = require("node:path");
const { syncVersions } = require("./sync-version.cjs");

const files = [
  "package.json",
  "package-lock.json",
  "src-tauri/Cargo.toml",
  "src-tauri/Cargo.lock",
  "src-tauri/tauri.conf.json",
];

function getVersion(root) {
  const pkg = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));
  if (typeof pkg.version !== "string") throw new Error("Missing app version in package.json.");
  return pkg.version;
}

function nextVersion(current, size) {
  if (!["big", "medium", "small"].includes(size)) {
    throw new Error("Choose big, medium, or small: npm.cmd run version:bump -- small");
  }
  const match = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.exec(current);
  if (!match) throw new Error("The current version must have three numeric parts, such as 1.4.3.");
  const [, major, minor, patch] = match.map(Number);
  if (size === "big") return `${major + 1}.0.0`;
  if (size === "medium") return `${major}.${minor + 1}.0`;
  return `${major}.${minor}.${patch + 1}`;
}

function bumpVersion(root, size) {
  const current = getVersion(root);
  const next = nextVersion(current, size);
  const original = new Map(files.map(file => [file, fs.readFileSync(path.join(root, file), "utf8")]));
  try {
    const filename = path.join(root, "package.json");
    const pkg = JSON.parse(original.get("package.json"));
    pkg.version = next;
    const newline = original.get("package.json").includes("\r\n") ? "\r\n" : "\n";
    fs.writeFileSync(filename, (JSON.stringify(pkg, null, 2) + "\n").replace(/\n/g, newline), "utf8");
    syncVersions(root);
  } catch (error) {
    for (const [file, contents] of original) fs.writeFileSync(path.join(root, file), contents, "utf8");
    throw error;
  }
  return { current, next };
}

module.exports = { getVersion, nextVersion, bumpVersion };
if (require.main === module) {
  try {
    const root = path.resolve(__dirname, "..");
    const [action, size, ...extra] = process.argv.slice(2);
    if (action === "get" && !size && !extra.length) {
      console.log(`Current version: ${getVersion(root)}`);
    } else if (action === "bump" && !extra.length) {
      const { current, next } = bumpVersion(root, size);
      console.log(`Version changed from ${current} to ${next}.`);
    } else {
      throw new Error("Use 'version:get' or 'version:bump -- big|medium|small'.");
    }
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
