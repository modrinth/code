import { execFileSync, execSync } from 'child_process'
import chalk from 'chalk'
import * as fs from 'fs'
import * as path from 'path'
import { fileURLToPath } from 'url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))

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

const PRODUCT_SUMMARY_MAP: Record<string, Product> = {
	App: 'app',
	Website: 'web',
	Hosting: 'hosting',
}

const GITHUB_API = 'https://api.github.com'
const REPO = 'modrinth/code'

interface ParsedChangelog {
	entries: Map<Product, Map<string, string[]>>
}

interface PRInfo {
	number: number
	mergedAt: string | null
	headRef: string
	headRepo: string | null
	stackedOn?: number
}

interface GitHubPR {
	number: number
	merged_at: string | null
	updated_at: string
	head: { ref: string; repo: { full_name: string } | null }
}

interface CommentInfo {
	id: number
	body: string
}

interface AppRelease {
	tag: string
	date: string
}

function parseArgs(argv: string[]): {
	version?: string
	dryRun: boolean
	pr?: number
} {
	let version: string | undefined
	let dryRun = false
	let pr: number | undefined

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
		} else if (argv[i] === '--pr') {
			i++
			if (i >= argv.length || argv[i].startsWith('--')) {
				console.error(chalk.red('--pr requires a value'))
				process.exit(1)
			}
			pr = parseInt(argv[i], 10)
			if (isNaN(pr)) {
				console.error(chalk.red('--pr must be a number'))
				process.exit(1)
			}
			i++
		} else {
			console.error(chalk.red(`Unknown argument: ${argv[i]}`))
			process.exit(1)
		}
	}

	return { version, dryRun, pr }
}

async function getParser() {
	const mod = await import('keep-a-changelog')
	return mod.parser
}

function indentSubBulletsWithTabs(markdown: string): string {
	const lines = markdown.split(/\r?\n/)
	const subBullet = /^([ \t]+)(?=[-*] )/
	const spaceWidths = lines.flatMap((line) => line.match(/^( +)[-*] /)?.[1].length ?? [])
	const unit = spaceWidths.length > 0 ? Math.min(...spaceWidths) : 2
	return lines
		.map((line) =>
			line.replace(subBullet, (indent) => {
				const tabs = indent.split('\t').length - 1
				const spaces = indent.length - tabs
				return '  ' + '\t'.repeat(tabs + Math.floor(spaces / unit))
			}),
		)
		.join('\n')
}

function parseChangelogComment(body: string, parse: Function): ParsedChangelog | null {
	if (!body.includes(CHANGELOG_MARKER)) return null

	const entries = new Map<Product, Map<string, string[]>>()

	const detailsRegex = /<details>\s*<summary>(.*?)<\/summary>([\s\S]*?)<\/details>/g
	let match
	while ((match = detailsRegex.exec(body)) !== null) {
		const summaryLabel = match[1].trim()
		const product = PRODUCT_SUMMARY_MAP[summaryLabel]
		if (!product) continue

		const content = match[2].replace(/<!--[\s\S]*?-->/g, '').trim()
		const firstSection = content.search(/^### /m)
		if (firstSection === -1) continue

		const changelogMd = `# Changelog\n\n## [Unreleased]\n${indentSubBulletsWithTabs(content.slice(firstSection))}`

		let changelog
		try {
			changelog = parse(changelogMd)
		} catch {
			continue
		}

		const unreleased = changelog.findRelease()
		if (!unreleased || unreleased.isEmpty()) continue

		const sections = new Map<string, string[]>()
		for (const type of SECTION_ORDER) {
			const changes = unreleased.changes.get(type)
			if (changes && changes.length > 0) {
				sections.set(
					type,
					changes.map((c: { title: string }) => c.title),
				)
			}
		}

		if (sections.size > 0) {
			entries.set(product, sections)
		}
	}

	if (entries.size === 0) return null

	return { entries }
}

function linkifyIssues(text: string): string {
	return text.replace(/\[#(\d+)\]/g, '[#$1](https://github.com/modrinth/code/issues/$1)')
}

function buildBody(sections: Map<string, string[]>): string {
	const parts: string[] = []

	for (const type of SECTION_ORDER) {
		const entries = sections.get(type)
		if (!entries || entries.length === 0) continue

		const lines = entries.map((e) => `- ${linkifyIssues(e)}`)
		parts.push(`${SECTION_HEADERS[type]}\n${lines.join('\n')}`)
	}

	return parts.join('\n\n').replace(/`/g, '\\`').replace(/\$/g, '\\$')
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

async function githubFetch(
	endpoint: string,
	token: string,
	options?: RequestInit,
): Promise<Response> {
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

		const data = (await res.json()) as GitHubPR[]

		if (data.length === 0) break

		let allTooOld = true
		for (const pr of data) {
			if (new Date(pr.updated_at) < new Date(sinceDate)) continue
			allTooOld = false
			if (!pr.merged_at) continue
			if (new Date(pr.merged_at) > new Date(sinceDate)) {
				prs.push(toPRInfo(pr))
			}
		}

		if (allTooOld || data.length < 100) break
		page++
	}

	return prs
}

function toPRInfo(pr: GitHubPR, stackedOn?: number): PRInfo {
	return {
		number: pr.number,
		mergedAt: pr.merged_at,
		headRef: pr.head.ref,
		headRepo: pr.head.repo?.full_name ?? null,
		stackedOn,
	}
}

async function fetchPR(token: string, prNumber: number): Promise<PRInfo> {
	const res = await githubFetch(`/repos/${REPO}/pulls/${prNumber}`, token)

	if (!res.ok) {
		console.error(
			chalk.red(`GitHub API error for PR #${prNumber}: ${res.status} ${res.statusText}`),
		)
		process.exit(1)
	}

	return toPRInfo((await res.json()) as GitHubPR)
}

async function fetchStackedPRs(
	token: string,
	parent: PRInfo,
	seen: Set<number>,
): Promise<PRInfo[]> {
	if (parent.headRepo !== REPO) return []

	const stacked: PRInfo[] = []
	let page = 1

	while (true) {
		const res = await githubFetch(
			`/repos/${REPO}/pulls?state=closed&base=${encodeURIComponent(parent.headRef)}&per_page=100&page=${page}`,
			token,
		)

		if (!res.ok) {
			console.error(chalk.red(`GitHub API error: ${res.status} ${res.statusText}`))
			process.exit(1)
		}

		const data = (await res.json()) as GitHubPR[]

		for (const pr of data) {
			if (!pr.merged_at || seen.has(pr.number)) continue
			if (parent.mergedAt && new Date(pr.merged_at) > new Date(parent.mergedAt)) continue

			seen.add(pr.number)
			const child = toPRInfo(pr, parent.number)
			stacked.push(child, ...(await fetchStackedPRs(token, child, seen)))
		}

		if (data.length < 100) break
		page++
	}

	return stacked
}

async function withStackedPRs(token: string, roots: PRInfo[]): Promise<PRInfo[]> {
	const seen = new Set(roots.map((pr) => pr.number))
	const prs: PRInfo[] = []

	for (const root of roots) {
		prs.push(root, ...(await fetchStackedPRs(token, root, seen)))
	}

	const stackedCount = prs.length - roots.length
	if (stackedCount > 0) {
		console.log(chalk.gray(`Found ${stackedCount} PR(s) merged into stacked branches`))
	}

	return prs
}

function getLatestAppRelease(): AppRelease {
	const tag = execFileSync(
		'git',
		['describe', '--tags', '--abbrev=0', '--match', 'v[0-9]*', 'origin/main'],
		{ encoding: 'utf-8' },
	).trim()
	const date = execFileSync('git', ['log', '-1', '--format=%cI', tag], {
		encoding: 'utf-8',
	}).trim()

	return { tag, date }
}

async function fetchBotComment(token: string, prNumber: number): Promise<CommentInfo | null> {
	const res = await githubFetch(`/repos/${REPO}/issues/${prNumber}/comments?per_page=100`, token)

	if (!res.ok) return null

	const comments = (await res.json()) as Array<{ id: number; body: string }>
	const comment = comments.find((c) => c.body.includes(CHANGELOG_MARKER))

	if (!comment) return null
	return { id: comment.id, body: comment.body }
}

async function markCommentBaked(
	token: string,
	commentId: number,
	currentBody: string,
): Promise<void> {
	const admonition =
		'> [!NOTE]\n> This changelog has been baked. Any further edits will not be reflected.\n\n'
	const newBody = admonition + currentBody

	await githubFetch(`/repos/${REPO}/issues/comments/${commentId}`, token, {
		method: 'PATCH',
		body: JSON.stringify({ body: newBody }),
	})
}

function getCurrentUTCISO(): string {
	return new Date().toISOString().replace(/\.\d{3}Z$/, '+00:00')
}

function generateEntry(product: string, body: string, version?: string): string {
	const dateStr = getCurrentUTCISO()
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
			console.error(
				chalk.red(
					'GITHUB_TOKEN not set and `gh auth token` failed. Run `gh auth login` or set GITHUB_TOKEN.',
				),
			)
			process.exit(1)
		}
	}

	const parse = await getParser()
	const rootDir = path.resolve(__dirname, '..')
	const changelogPath = path.join(rootDir, 'packages', 'blog', 'changelog.ts')

	let roots: PRInfo[]

	if (args.pr) {
		console.log(chalk.gray(`Targeting single PR #${args.pr}`))
		roots = [await fetchPR(token, args.pr)]
	} else {
		const prodDate = execSync('git log -1 --format=%aI origin/prod', { encoding: 'utf-8' }).trim()
		console.log(chalk.gray(`Last prod commit: ${prodDate}`))

		let sinceDate = prodDate
		if (args.version) {
			const appRelease = getLatestAppRelease()
			console.log(chalk.gray(`Last app release: ${appRelease.tag} (${appRelease.date})`))
			if (new Date(appRelease.date) < new Date(sinceDate)) {
				sinceDate = appRelease.date
			}
		}

		roots = await fetchMergedPRs(token, sinceDate)
		console.log(chalk.gray(`Found ${roots.length} merged PR(s) since ${sinceDate}`))

		if (roots.length === 0) {
			console.log(chalk.yellow(`No merged PRs found since ${sinceDate}`))
			return
		}
	}

	const prs = await withStackedPRs(token, roots)

	const allEntries = new Map<Product, { sections: Map<string, string[]>; prNumber: number }[]>()
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

		const products = [...parsed.entries.keys()]
		const types = [...new Set([...parsed.entries.values()].flatMap((s) => [...s.keys()]))]
		const label = pr.stackedOn
			? `PR #${pr.number} (stacked on #${pr.stackedOn})`
			: `PR #${pr.number}`
		console.log(chalk.cyan(`${label}: ${products.join(', ')} — ${types.join(', ')}`))

		for (const [product, sections] of parsed.entries) {
			if (!allEntries.has(product)) {
				allEntries.set(product, [])
			}
			allEntries.get(product)!.push({ sections, prNumber: pr.number })
		}

		processedComments.push({ commentId: comment.id, body: comment.body })
	}

	if (allEntries.size === 0) {
		console.log(chalk.yellow('No changelog entries found in merged PRs'))
		return
	}

	const hasApp = allEntries.has('app')
	if (hasApp && !args.version) {
		console.error(chalk.red('--version is required when app changelog entries exist'))
		process.exit(1)
	}

	const products = [...allEntries.keys()].reverse()
	for (const product of products) {
		const productEntries = allEntries.get(product)!
		const mergedSections = mergeSections(productEntries)
		const body = buildBody(mergedSections)

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
