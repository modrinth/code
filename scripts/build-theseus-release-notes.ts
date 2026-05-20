/**
 * Builds merged app + Modrinth Hosting release notes for GitHub releases and Tauri updates.json.
 * Hosting bullets are folded into each ## section as `- **Modrinth Hosting:** …`.
 *
 * Hosting-only sections (present in hosting changelog but not app) are appended after app sections,
 * ordered by: added, changed, deprecated, removed, fixed, security, then other titles alphabetically.
 *
 * Run locally: `pnpm changelog:combine-for-app -- --dry-run 0.13.2` or `pnpm scripts build-theseus-release-notes -- ...`
 */

import * as fs from 'fs'
import { dirname, join } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const REPO_ROOT = join(__dirname, '..')

type Product = 'web' | 'app' | 'hosting'

interface ChangelogEntry {
	date: string
	product: Product
	version: string | undefined
	body: string
}

interface ParsedSection {
	key: string
	title: string
	rawLines: string[]
}

interface HostingSectionAgg {
	title: string
	bullets: string[]
}

// Mirror scripts/collect-changelog.ts — used to order hosting-only sections at the end
const KNOWN_SECTION_ORDER = ['added', 'changed', 'deprecated', 'removed', 'fixed', 'security'] as const

function parseArgs(argv: string[]): { dryRun: boolean; version: string; outFile: string } {
	let dryRun = false
	let dryRunVersion: string | undefined
	let version = process.env.VERSION ? process.env.VERSION.replace(/^v/, '') : undefined

	for (let i = 0; i < argv.length; i++) {
		const a = argv[i]
		if (a === '--dry-run') {
			dryRun = true
			dryRunVersion = argv[++i]
			if (!dryRunVersion || dryRunVersion.startsWith('--')) {
				console.error('Usage: --dry-run <version>')
				process.exit(1)
			}
		} else if (a === '--version') {
			const v = argv[++i]
			if (!v || v.startsWith('--')) {
				console.error('--version requires a value')
				process.exit(1)
			}
			version = v.replace(/^v/, '')
		}
	}

	if (dryRun) {
		version = dryRunVersion!.replace(/^v/, '')
	} else if (!version) {
		console.error('Set VERSION (e.g. from tag) or pass --version')
		process.exit(1)
	}

	const outFile = dryRun ? join(REPO_ROOT, 'test_result.md') : join(process.cwd(), 'release-notes.md')
	return { dryRun, version, outFile }
}

/**
 * Parse every entry in changelog.ts VERSIONS (reverse chronological order).
 */
function parseChangelogEntries(src: string): ChangelogEntry[] {
	const entryRe =
		/\{\s*date:\s*`([^`]+)`,\s*product:\s*'(\w+)',(?:\s*version:\s*[`']([^`']+)[`'],)?\s*body:\s*`([\s\S]*?)`,\s*\}/g
	const entries: ChangelogEntry[] = []
	let match: RegExpExecArray | null
	while ((match = entryRe.exec(src)) !== null) {
		entries.push({
			date: match[1],
			product: match[2] as Product,
			version: match[3],
			body: match[4],
		})
	}
	return entries
}

function findAppAndHosting(entries: ChangelogEntry[], version: string): { appBody: string; hostingEntries: ChangelogEntry[] } {
	const currentIdx = entries.findIndex((e) => e.product === 'app' && e.version === version)
	if (currentIdx === -1) {
		throw new Error(`No app changelog entry found for version ${version}`)
	}

	let newerAppIdx = -1
	for (let i = currentIdx - 1; i >= 0; i--) {
		if (entries[i].product === 'app') {
			newerAppIdx = i
			break
		}
	}
	let previousAppIdx = entries.length
	for (let i = currentIdx + 1; i < entries.length; i++) {
		if (entries[i].product === 'app') {
			previousAppIdx = i
			break
		}
	}

	const hostingEntries: ChangelogEntry[] = []
	for (let i = newerAppIdx + 1; i < previousAppIdx; i++) {
		if (entries[i].product === 'hosting') {
			hostingEntries.push(entries[i])
		}
	}

	return {
		appBody: entries[currentIdx].body,
		hostingEntries,
	}
}

function parseSections(markdown: string): ParsedSection[] {
	const lines = markdown.split('\n')
	const sections: ParsedSection[] = []
	let current: ParsedSection | null = null

	for (const line of lines) {
		const m = line.match(/^## (.+)$/)
		if (m) {
			const title = m[1].trim()
			const key = title.toLowerCase()
			current = { key, title, rawLines: [] }
			sections.push(current)
		} else if (current) {
			current.rawLines.push(line)
		}
	}
	return sections
}

function extractBulletLines(rawLines: string[]): string[] {
	const out: string[] = []
	for (const line of rawLines) {
		if (/^\s*-\s/.test(line)) {
			out.push(line.trim())
		}
	}
	return out
}

function toHostingBullet(line: string): string {
	const m = line.match(/^\s*-\s(.*)$/)
	const rest = m ? m[1].trim() : line.trim()
	return `- **Modrinth Hosting:** ${rest}`
}

function sortHostingOnlyKeys(keys: string[]): string[] {
	return [...keys].sort((a, b) => {
		const ia = KNOWN_SECTION_ORDER.indexOf(a as (typeof KNOWN_SECTION_ORDER)[number])
		const ib = KNOWN_SECTION_ORDER.indexOf(b as (typeof KNOWN_SECTION_ORDER)[number])
		const aKnown = ia !== -1
		const bKnown = ib !== -1
		if (aKnown && bKnown) return ia - ib
		if (aKnown) return -1
		if (bKnown) return 1
		return a.localeCompare(b)
	})
}

function mergeAppAndHosting(appBody: string, hostingEntries: ChangelogEntry[]): string {
	if (!hostingEntries.length) {
		return appBody.replace(/\s*$/, '\n')
	}

	const appSections = parseSections(appBody)
	const hostingByKey = new Map<string, HostingSectionAgg>()

	for (const entry of hostingEntries) {
		for (const sec of parseSections(entry.body)) {
			const bullets = extractBulletLines(sec.rawLines).map(toHostingBullet)
			if (bullets.length === 0) continue

			if (!hostingByKey.has(sec.key)) {
				hostingByKey.set(sec.key, { title: sec.title, bullets: [] })
			}
			hostingByKey.get(sec.key)!.bullets.push(...bullets)
		}
	}

	const parts: string[] = []

	for (const sec of appSections) {
		const appBullets = extractBulletLines(sec.rawLines)
		const hostBlock = hostingByKey.get(sec.key)
		const hostBullets = hostBlock ? hostBlock.bullets : []
		if (hostBlock) {
			hostingByKey.delete(sec.key)
		}

		const lines = [`## ${sec.title}`, '', ...appBullets, ...hostBullets]
		parts.push(lines.join('\n'))
	}

	for (const key of sortHostingOnlyKeys([...hostingByKey.keys()])) {
		const block = hostingByKey.get(key)!
		parts.push(`## ${block.title}\n\n${block.bullets.join('\n')}`)
	}

	return `${parts.join('\n\n')}\n`
}

function main() {
	const { dryRun, version, outFile } = parseArgs(process.argv.slice(2))
	const changelogPath = join(REPO_ROOT, 'packages/blog/changelog.ts')
	const src = fs.readFileSync(changelogPath, 'utf8')
	const entries = parseChangelogEntries(src)
	const { appBody, hostingEntries } = findAppAndHosting(entries, version)
	const output = mergeAppAndHosting(appBody, hostingEntries)

	fs.writeFileSync(outFile, output, 'utf8')
	const mode = dryRun ? 'dry-run' : 'release'
	const n = hostingEntries.length
	console.log(`Wrote ${outFile} (${mode}, app ${version}, ${n} hosting entr${n === 1 ? 'y' : 'ies'})`)
}

main()
