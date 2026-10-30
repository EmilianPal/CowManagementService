const fs = require("node:fs");
const path = require("node:path");

function updateBlockVersion(source, boundary, matches, version, filename) {
  const blocks = source.split(boundary);
  const indexes = blocks.flatMap((block, index) => matches(block) ? [index] : []);
  if (indexes.length !== 1) throw new Error(`Cannot identify the app package in ${filename}.`);
  const index = indexes[0];
  const pattern = /^(version\s*=\s*")[^"]*(")/m;
  if (!pattern.test(blocks[index])) throw new Error(`Missing version in ${filename}.`);
  blocks[index] = blocks[index].replace(pattern, (_, before, after) => before + version + after);
  return blocks.join("");
}

function syncVersions(root) {
  const read = filename => fs.readFileSync(path.join(root, filename), "utf8");
  const pkg = JSON.parse(read("package.json"));
  const version = pkg.version;
  if (typeof version !== "string" || !/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/.test(version)) {
    throw new Error("package.json must contain a valid app version, e.g. 1.0.2.");
  }
  const files = ["src-tauri/Cargo.toml", "src-tauri/Cargo.lock", "src-tauri/tauri.conf.json", "package-lock.json"];
  const original = Object.fromEntries(files.map(file => [file, read(file)]));
  const next = { ...original };
  next[files[0]] = updateBlockVersion(original[files[0]], /(?=^\[)/m,
    block => /^\[package\][ \t]*\r?$/m.test(block), version, files[0]);
  next[files[1]] = updateBlockVersion(original[files[1]], /(?=^\[\[package\]\])/m,
    block => block.split(/\r?\n/).some(line => line === `name = "${pkg.name}"`), version, files[1]);

  const config = JSON.parse(original[files[2]]);
  if (typeof config.version !== "string") throw new Error("Missing Tauri version.");
  next[files[2]] = original[files[2]].replace(/("version"\s*:\s*")[^"]*(")/, (_, before, after) => before + version + after);

  const lock = JSON.parse(original[files[3]]);
  if (!lock.packages?.[""] || lock.name !== pkg.name) throw new Error("Unexpected npm lockfile structure.");
  lock.version = version;
  lock.packages[""].version = version;
  const newline = original[files[3]].includes("\r\n") ? "\r\n" : "\n";
  next[files[3]] = (JSON.stringify(lock, null, 2) + "\n").replace(/\n/g, newline);

  // Validate every file before writing; restore previous contents on write failures.
  const written = [];
  try {
    for (const file of files) {
      if (next[file] === original[file]) continue;
      written.push(file);
      fs.writeFileSync(path.join(root, file), next[file], "utf8");
    }
  } catch (error) {
    for (const file of written.reverse()) fs.writeFileSync(path.join(root, file), original[file], "utf8");
    throw error;
  }
  return version;
}

module.exports = { syncVersions };
if (require.main === module) {
  try {
    console.log(`App version synchronized: ${syncVersions(path.resolve(__dirname, ".."))}`);
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
