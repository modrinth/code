#!/usr/bin/env node
/**
 * Patch launcher semver for release CI (Cargo + app-frontend package.json).
 * Idempotent: OK if version is already set.
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
const cargoMatch = cargoText.match(/^version = "(.*)"$/m)
if (!cargoMatch) {
	console.error(`No version = "..." line in ${cargoFile}`)
	process.exit(1)
}
if (cargoMatch[1] === version) {
	console.log(`${cargoFile} already ${version}`)
} else {
	const cargoNext = cargoText.replace(/^version = ".*"$/m, `version = "${version}"`)
	fs.writeFileSync(cargoFile, cargoNext)
	console.log(`Patched ${cargoFile} -> ${version}`)
}

const pkgFile = path.join('apps', 'app-frontend', 'package.json')
const pkg = JSON.parse(fs.readFileSync(pkgFile, 'utf8'))
if (pkg.version === version) {
	console.log(`${pkgFile} already ${version}`)
} else {
	pkg.version = version
	fs.writeFileSync(pkgFile, `${JSON.stringify(pkg, null, '\t')}\n`)
	console.log(`Patched ${pkgFile} -> ${version}`)
}
