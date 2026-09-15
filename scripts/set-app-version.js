#!/usr/bin/env node
/**
 * Patch launcher semver for release CI (Cargo + app-frontend package.json).
 * Usage: node scripts/set-app-version.js 0.2.0
 */
const fs = require('fs')
const path = require('path')

const version = process.argv[2] || process.env.V
if (!version || !/^\d+\.\d+\.\d+[0-9A-Za-z.+-]*$/.test(version)) {
	console.error('Usage: node scripts/set-app-version.js <semver>')
	process.exit(1)
}

const cargoFile = path.join('apps', 'app', 'Cargo.toml')
const cargoText = fs.readFileSync(cargoFile, 'utf8')
const cargoNext = cargoText.replace(/^version = ".*"$/m, `version = "${version}"`)
if (cargoNext === cargoText) {
	console.error(`Failed to patch version in ${cargoFile}`)
	process.exit(1)
}
fs.writeFileSync(cargoFile, cargoNext)
console.log(`Patched ${cargoFile} -> ${version}`)

const pkgFile = path.join('apps', 'app-frontend', 'package.json')
const pkg = JSON.parse(fs.readFileSync(pkgFile, 'utf8'))
pkg.version = version
fs.writeFileSync(pkgFile, `${JSON.stringify(pkg, null, '\t')}\n`)
console.log(`Patched ${pkgFile} -> ${version}`)
