import { execSync } from 'child_process'
import chalk from 'chalk'
import * as fs from 'fs'
import * as path from 'path'

type Product = 'web' | 'app' | 'hosting'

const CHANGELOG_MARKER = '<!-- changelog -->'
const SECTION_ORDER = ['added', 'changed', 'deprecated', 'removed', 'fixed', 'security'] as const
const SECTION_HEADERS: Record<string, string> = {
	added: '## Added',
	changed: '## Changed',
	deprecated: '## Deprecated',
	removed: '## Removed',
	fixed: '## Fixed',
	security: '## Security',
}

const PRODUCT_CHECKBOX_MAP: Record<string, Product> = {
	App: 'app',
	Website: 'web',
	Hosting: 'hosting',
}

const GITHUB_API = 'https://api.github.com'
const REPO = 'modrinth/code'

interface ParsedChangelog {
	products: Product[]
	sections: Map<string, string[]>
}

interface PRInfo {
	number: number
	mergedAt: string
}

interface CommentInfo {
	id: number
	body: string
}

function parseArgs(argv: string[]): {
	version?: string
	dryRun: boolean
} {
	let version: string | undefined
	let dryRun = false

	let i = 0
	while (i < argv.length) {
		if (argv[i] === '--') {
			i++
			continue
		}
		if (argv[i] === '--version') {
			i++
			if (i >= argv.length || argv[i].startsWith('--')) {
				console.error(chalk.red('--version requires a value'))
				process.exit(1)
			}
			version = argv[i]
			i++
		} else if (argv[i] === '--dry-run') {
			dryRun = true
			i++
		} else {
			console.error(chalk.red(`Unknown argument: ${argv[i]}`))
			process.exit(1)
		}
	}

	return { version, dryRun }
}

async function getParser() {
	const mod = await import('keep-a-changelog')
	return mod.parser
}

function parseChangelogComment(body: string, parse: Function): ParsedChangelog | null {
	if (!body.includes(CHANGELOG_MARKER)) return null

	const products: Product[] = []
	for (const [label, product] of Object.entries(PRODUCT_CHECKBOX_MAP)) {
		if (new RegExp(`- \\[x\\] ${label}`, 'i').test(body)) {
			products.push(product)
		}
	}

	if (/- \[x\] No changelog/i.test(body)) return null
	if (products.length === 0) return null

	const sectionContent = body.replace(/<!--[\s\S]*?-->/g, '')
	const firstSection = sectionContent.search(/^### /m)
	if (firstSection === -1) return null

	const changelogMd = `# Changelog\n\n## [Unreleased]\n${sectionContent.slice(firstSection)}`

	let changelog
	try {
		changelog = parse(changelogMd)
	} catch {
		return null
	}

	const unreleased = changelog.findRelease()
	if (!unreleased || unreleased.isEmpty()) return null

	const sections = new Map<string, string[]>()
	for (const type of SECTION_ORDER) {
		const changes = unreleased.changes.get(type)
		if (changes && changes.length > 0) {
			sections.set(type, changes.map((c: { title: string }) => c.title))
		}
	}

	if (sections.size === 0) return null

	return { products, sections }
}

function buildBody(sections: Map<string, string[]>): string {
	const parts: string[] = []

	for (const type of SECTION_ORDER) {
		const entries = sections.get(type)
		if (!entries || entries.length === 0) continue

		const lines = entries.map((e) => `- ${e}`)
		parts.push(`${SECTION_HEADERS[type]}\n${lines.join('\n')}`)
	}

	return parts.join('\n\n')
}

function mergeSections(
	allEntries: { sections: Map<string, string[]>; prNumber: number }[],
): Map<string, string[]> {
	const merged = new Map<string, string[]>()

	for (const { sections } of allEntries) {
		for (const [type, entries] of sections) {
			if (!merged.has(type)) {
				merged.set(type, [])
			}
			merged.get(type)!.push(...entries)
		}
	}

	return merged
}

async function githubFetch(endpoint: string, token: string, options?: RequestInit): Promise<Response> {
	const url = endpoint.startsWith('http') ? endpoint : `${GITHUB_API}${endpoint}`
	return fetch(url, {
		...options,
		headers: {
			Authorization: `token ${token}`,
			Accept: 'application/vnd.github.v3+json',
			...options?.headers,
		},
	})
}

async function fetchMergedPRs(token: string, sinceDate: string): Promise<PRInfo[]> {
	const prs: PRInfo[] = []
	let page = 1

	while (true) {
		const res = await githubFetch(
			`/repos/${REPO}/pulls?state=closed&base=main&sort=updated&direction=desc&per_page=100&page=${page}`,
			token,
		)

		if (!res.ok) {
			console.error(chalk.red(`GitHub API error: ${res.status} ${res.statusText}`))
			process.exit(1)
		}

		const data = (await res.json()) as Array<{
			number: number
			merged_at: string | null
			updated_at: string
		}>

		if (data.length === 0) break

		let foundOlder = false
		for (const pr of data) {
			if (!pr.merged_at) continue

			if (new Date(pr.merged_at) > new Date(sinceDate)) {
				prs.push({ number: pr.number, mergedAt: pr.merged_at })
			} else {
				foundOlder = true
			}
		}

		if (foundOlder || data.length < 100) break
		page++
	}

	return prs
}

async function fetchBotComment(token: string, prNumber: number): Promise<CommentInfo | null> {
	const res = await githubFetch(
		`/repos/${REPO}/issues/${prNumber}/comments?per_page=100`,
		token,
	)

	if (!res.ok) return null

	const comments = (await res.json()) as Array<{ id: number; body: string }>
	const comment = comments.find((c) => c.body.includes(CHANGELOG_MARKER))

	if (!comment) return null
	return { id: comment.id, body: comment.body }
}

async function markCommentBaked(token: string, commentId: number, currentBody: string): Promise<void> {
	const admonition = '> [!NOTE]\n> This changelog has been baked. Any further edits will not be reflected.\n\n'
	const newBody = admonition + currentBody

	await githubFetch(`/repos/${REPO}/issues/comments/${commentId}`, token, {
		method: 'PATCH',
		body: JSON.stringify({ body: newBody }),
	})
}

function getCurrentPacificISO(): string {
	const now = new Date()

	const formatter = new Intl.DateTimeFormat('en-US', {
		timeZone: 'America/Los_Angeles',
		year: 'numeric',
		month: '2-digit',
		day: '2-digit',
		hour: '2-digit',
		minute: '2-digit',
		second: '2-digit',
		hour12: false,
	})

	const parts = formatter.formatToParts(now)
	const get = (type: string) => parts.find((p) => p.type === type)?.value ?? '00'

	const year = get('year')
	const month = get('month')
	const day = get('day')
	const hour = get('hour')
	const minute = get('minute')
	const second = get('second')

	const jan = new Date(now.getFullYear(), 0, 1)
	const jul = new Date(now.getFullYear(), 6, 1)
	const stdOffset = Math.max(jan.getTimezoneOffset(), jul.getTimezoneOffset())
	const pacificNow = new Date(
		now.toLocaleString('en-US', { timeZone: 'America/Los_Angeles' }),
	)
	const utcNow = new Date(now.toLocaleString('en-US', { timeZone: 'UTC' }))
	const offsetMinutes = (utcNow.getTime() - pacificNow.getTime()) / 60000
	const isDST = offsetMinutes !== stdOffset

	const offsetStr = isDST ? '-07:00' : '-08:00'

	return `${year}-${month}-${day}T${hour}:${minute}:${second}${offsetStr}`
}

function generateEntry(product: string, body: string, version?: string): string {
	const dateStr = getCurrentPacificISO()
	const versionLine = version ? `\n\t\tversion: '${version}',` : ''

	return `\t{
\t\tdate: \`${dateStr}\`,
\t\tproduct: '${product}',${versionLine}
\t\tbody: \`${body}\`,
\t},`
}

function insertIntoChangelog(changelogPath: string, entryString: string): void {
	const content = fs.readFileSync(changelogPath, 'utf-8')

	const marker = 'const VERSIONS: VersionEntry[] = ['
	const markerIndex = content.indexOf(marker)

	if (markerIndex === -1) {
		console.error(chalk.red('Could not find VERSIONS array in changelog.ts'))
		process.exit(1)
	}

	const insertPos = content.indexOf('\n', markerIndex) + 1

	const newContent = content.slice(0, insertPos) + entryString + '\n' + content.slice(insertPos)

	fs.writeFileSync(changelogPath, newContent, 'utf-8')
}

async function main() {
	const args = parseArgs(process.argv.slice(2))

	let token = process.env.GITHUB_TOKEN
	if (!token) {
		try {
			token = execSync('gh auth token', { encoding: 'utf-8' }).trim()
		} catch {
			console.error(chalk.red('GITHUB_TOKEN not set and `gh auth token` failed. Run `gh auth login` or set GITHUB_TOKEN.'))
			process.exit(1)
		}
	}

	const parse = await getParser()
	const rootDir = path.resolve(__dirname, '..')
	const changelogPath = path.join(rootDir, 'packages', 'blog', 'changelog.ts')

	const prodDate = execSync('git log -1 --format=%aI origin/prod', { encoding: 'utf-8' }).trim()
	console.log(chalk.gray(`Last prod commit: ${prodDate}`))

	const prs = await fetchMergedPRs(token, prodDate)
	console.log(chalk.gray(`Found ${prs.length} merged PR(s) since last prod deploy`))

	if (prs.length === 0) {
		console.log(chalk.yellow('No merged PRs found since last prod deploy'))
		return
	}

	const entries = new Map<Product, { sections: Map<string, string[]>; prNumber: number }[]>()
	const processedComments: { commentId: number; body: string }[] = []

	for (const pr of prs) {
		const comment = await fetchBotComment(token, pr.number)
		if (!comment) {
			console.log(chalk.gray(`PR #${pr.number}: no changelog comment, skipping`))
			continue
		}

		if (comment.body.includes('> This changelog has been baked.')) {
			console.log(chalk.gray(`PR #${pr.number}: already baked, skipping`))
			continue
		}

		const parsed = parseChangelogComment(comment.body, parse)
		if (!parsed) {
			console.log(chalk.gray(`PR #${pr.number}: no changelog entries, skipping`))
			continue
		}

		console.log(
			chalk.cyan(
				`PR #${pr.number}: ${parsed.products.join(', ')} — ${[...parsed.sections.keys()].join(', ')}`,
			),
		)

		for (const product of parsed.products) {
			if (!entries.has(product)) {
				entries.set(product, [])
			}
			entries.get(product)!.push({ sections: parsed.sections, prNumber: pr.number })
		}

		processedComments.push({ commentId: comment.id, body: comment.body })
	}

	if (entries.size === 0) {
		console.log(chalk.yellow('No changelog entries found in merged PRs'))
		return
	}

	const hasApp = entries.has('app')
	if (hasApp && !args.version) {
		console.error(chalk.red('--version is required when app changelog entries exist'))
		process.exit(1)
	}

	const products = [...entries.keys()].reverse()
	for (const product of products) {
		const productEntries = entries.get(product)!
		const mergedSections = mergeSections(productEntries)
		const body = buildBody(mergedSections)

		if (body.includes('`')) {
			console.error(
				chalk.yellow(
					`Warning: Changelog body for ${product} contains backticks — this may break the template literal in changelog.ts`,
				),
			)
		}

		const version = product === 'app' ? args.version : undefined
		const entryString = generateEntry(product, body, version)

		if (args.dryRun) {
			console.log(chalk.cyan(`\n[dry-run] Would insert for ${product}:`))
			console.log(entryString)
		} else {
			insertIntoChangelog(changelogPath, entryString)
			console.log(chalk.green(`Inserted changelog entry for ${product}`))
		}
	}

	if (!args.dryRun) {
		for (const { commentId, body } of processedComments) {
			await markCommentBaked(token, commentId, body)
		}
		console.log(chalk.gray(`Marked ${processedComments.length} comment(s) as baked`))
	}

	console.log()
	if (args.dryRun) {
		console.log(chalk.cyan('Dry run complete — no changes written'))
	} else {
		console.log(chalk.green('Done! Review the changes in packages/blog/changelog.ts'))
	}
}

main()
