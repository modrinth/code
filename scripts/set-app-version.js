#!/usr/bin/env node
/**
 * Patch apps/app/Cargo.toml package version for release CI.
 * Usage: node scripts/set-app-version.js 0.1.0
 */
const fs = require('fs')
const path = require('path')

const version = process.argv[2] || process.env.V
if (!version || !/^\d+\.\d+\.\d+[0-9A-Za-z.+-]*$/.test(version)) {
	console.error('Usage: node scripts/set-app-version.js <semver>')
	process.exit(1)
}

const file = path.join('apps', 'app', 'Cargo.toml')
const text = fs.readFileSync(file, 'utf8')
const next = text.replace(/^version = ".*"$/m, `version = "${version}"`)
if (next === text) {
	console.error(`Failed to patch version in ${file}`)
	process.exit(1)
}
fs.writeFileSync(file, next)
console.log(`Patched ${file} -> ${version}`)
