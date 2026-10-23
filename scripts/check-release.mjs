import fs from 'node:fs';
import assert from 'node:assert/strict';
const version=JSON.parse(fs.readFileSync('package.json')).version;
assert.equal(process.argv[2],`v${version}`,'Tag must match package version');
assert.equal(JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json')).version,version);
assert.match(fs.readFileSync('src-tauri/Cargo.toml','utf8'),new RegExp(`version = "${version.replaceAll('.','\\.')}"`));
assert.match(fs.readFileSync('packaging/arch/PKGBUILD','utf8'),new RegExp(`pkgver=${version.replaceAll('.','\\.')}`));
console.log(`Release versions agree: ${version}`);
