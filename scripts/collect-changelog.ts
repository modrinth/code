import { execSync } from 'child_process'
import chalk from 'chalk'
import * as fs from 'fs'
import * as path from 'path'

type Product = 'web' | 'app' | 'hosting'

const CHANGELOG_MARKER = '<!-- changelog -->'
const BAKE_STATUS_MARKER = '<!-- changelog:bake-status -->'
const LEGACY_BAKED_NOTICE = '> This changelog has been baked.'
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

const PRODUCT_LABELS: Record<Product, string> = {
	app: 'App',
	web: 'Website',
	hosting: 'Hosting',
}

const PRODUCT_BAKE_MARKERS: Record<Product, string> = {
	app: '<!-- changelog:product:app -->',
	web: '<!-- changelog:product:web -->',
	hosting: '<!-- changelog:product:hosting -->',
}

const GITHUB_API = 'https://api.github.com'
const REPO = 'modrinth/code'

interface ParsedChangelog {
	entries: Map<Product, Map<string, string[]>>
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
	noBake: boolean
	pr?: number
} {
	let version: string | undefined
	let dryRun = false
	let noBake = false
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
		} else if (argv[i] === '--no-bake') {
			noBake = true
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

	return { version, dryRun, noBake, pr }
}

async function getParser() {
	const mod = await import('keep-a-changelog')
	return mod.parser
}

function getBakedProducts(body: string): Set<Product> {
	if (body.includes(LEGACY_BAKED_NOTICE)) {
		return new Set<Product>(['app', 'web', 'hosting'])
	}

	const baked = new Set<Product>()
	for (const line of body.split('\n')) {
		const normalized = line.replace(/^>\s?/, '').trim()

		for (const product of Object.keys(PRODUCT_BAKE_MARKERS) as Product[]) {
			if (!normalized.includes(PRODUCT_BAKE_MARKERS[product])) continue

			if (/^-\s+\[[xX]\]/.test(normalized)) {
				baked.add(product)
			}
		}
	}

	return baked
}

function buildBakeStatusBlock(bakedProducts: Set<Product>): string {
	return [
		BAKE_STATUS_MARKER,
		'> [!NOTE]',
		'> Changelog bake status:',
		...(['app', 'web', 'hosting'] as Product[]).map(
			(product) =>
				`> - [${bakedProducts.has(product) ? 'x' : ' '}] ${PRODUCT_LABELS[product]} ${PRODUCT_BAKE_MARKERS[product]}`,
		),
	].join('\n')
}

function stripLegacyBakeNotice(body: string): string {
	return body.replace(
		/^> \[!NOTE\]\n> This changelog has been baked\. Any further edits will not be reflected\.\n\n/,
		'',
	)
}

function setBakeStatusBlock(body: string, bakedProducts: Set<Product>): string {
	const bodyWithoutLegacyNotice = stripLegacyBakeNotice(body)
	const lines = bodyWithoutLegacyNotice.split('\n')
	const block = buildBakeStatusBlock(bakedProducts)
	const statusIndex = lines.findIndex((line) => line.includes(BAKE_STATUS_MARKER))

	if (statusIndex !== -1) {
		let endIndex = statusIndex + 1
		const seenProducts = new Set<Product>()

		while (endIndex < lines.length) {
			for (const product of Object.keys(PRODUCT_BAKE_MARKERS) as Product[]) {
				if (lines[endIndex].includes(PRODUCT_BAKE_MARKERS[product])) {
					seenProducts.add(product)
				}
			}

			endIndex++

			if (seenProducts.size === Object.keys(PRODUCT_BAKE_MARKERS).length) {
				if (lines[endIndex]?.trim() === '') {
					endIndex++
				}
				break
			}
		}

		lines.splice(statusIndex, endIndex - statusIndex, block, '')
		return lines.join('\n')
	}

	const markerIndex = lines.findIndex((line) => line.includes(CHANGELOG_MARKER))
	if (markerIndex === -1) {
		return [block, '', bodyWithoutLegacyNotice].join('\n')
	}

	let insertIndex = markerIndex + 1
	if (lines[insertIndex]?.startsWith('## Pull request changelog')) {
		insertIndex++
		if (lines[insertIndex]?.trim() === '') {
			insertIndex++
		}
	}
	lines.splice(insertIndex, 0, block, '')
	return lines.join('\n')
}

function parseChangelogComment(body: string, parse: Function): ParsedChangelog | null {
	if (!body.includes(CHANGELOG_MARKER)) return null

	const bakedProducts = getBakedProducts(body)
	const entries = new Map<Product, Map<string, string[]>>()

	const detailsRegex = /<details>\s*<summary>(.*?)<\/summary>([\s\S]*?)<\/details>/g
	let match
	while ((match = detailsRegex.exec(body)) !== null) {
		const summaryLabel = match[1].trim()
		const product = PRODUCT_SUMMARY_MAP[summaryLabel]
		if (!product) continue
		if (bakedProducts.has(product)) continue

		const content = match[2].replace(/<!--[\s\S]*?-->/g, '').trim()
		const firstSection = content.search(/^### /m)
		if (firstSection === -1) continue

		const changelogMd = `# Changelog\n\n## [Unreleased]\n${content.slice(firstSection)}`

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
				sections.set(type, changes.map((c: { title: string }) => c.title))
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
	return text.replace(
		/\[#(\d+)\]/g,
		'[#$1](https://github.com/modrinth/code/issues/$1)',
	)
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

		let allTooOld = true
		for (const pr of data) {
			if (new Date(pr.updated_at) < new Date(sinceDate)) continue
			allTooOld = false
			if (!pr.merged_at) continue
			if (new Date(pr.merged_at) > new Date(sinceDate)) {
				prs.push({ number: pr.number, mergedAt: pr.merged_at })
			}
		}

		if (allTooOld || data.length < 100) break
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

async function markCommentProductsBaked(
	token: string,
	commentId: number,
	currentBody: string,
	products: Set<Product>,
): Promise<void> {
	const bakedProducts = getBakedProducts(currentBody)
	for (const product of products) {
		bakedProducts.add(product)
	}

	const newBody = setBakeStatusBlock(currentBody, bakedProducts)

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
	const dateLine = product === 'app' && version ? '' : `\t\tdate: \`${dateStr}\`,\n`
	const versionLine = version ? `\n\t\tversion: '${version}',` : ''

	return `\t{
${dateLine}\t\tproduct: '${product}',${versionLine}
\t\tbody: \`${body}\`,
\t},`
}

function insertIntoChangelog(changelogPath: string, entryString: string): void {
	const content = fs.readFileSync(changelogPath, 'utf-8')

	const marker = 'const VERSIONS: ChangelogEntry[] = (['
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
	const targetProducts = new Set<Product>(args.version ? ['app', 'web', 'hosting'] : ['web', 'hosting'])

	let prs: PRInfo[]

	if (args.pr) {
		console.log(chalk.gray(`Targeting single PR #${args.pr}`))
		prs = [{ number: args.pr, mergedAt: '' }]
	} else {
		const prodDate = execSync('git log -1 --format=%aI origin/prod', { encoding: 'utf-8' }).trim()
		console.log(chalk.gray(`Last prod commit: ${prodDate}`))

		prs = await fetchMergedPRs(token, prodDate)
		console.log(chalk.gray(`Found ${prs.length} merged PR(s) since last prod deploy`))

		if (prs.length === 0) {
			console.log(chalk.yellow('No merged PRs found since last prod deploy'))
			return
		}
	}

	const allEntries = new Map<Product, { sections: Map<string, string[]>; prNumber: number }[]>()
	const processedComments = new Map<number, { body: string; products: Set<Product> }>()
	const unreleasedAppPRs: number[] = []

	for (const pr of prs) {
		const comment = await fetchBotComment(token, pr.number)
		if (!comment) {
			console.log(chalk.gray(`PR #${pr.number}: no changelog comment, skipping`))
			continue
		}

		if (comment.body.includes('> This changelog has been baked.')) {
			console.log(chalk.gray(`PR #${pr.number}: all changelog products already baked, skipping`))
			continue
		}

		const parsed = parseChangelogComment(comment.body, parse)
		if (!parsed) {
			console.log(chalk.gray(`PR #${pr.number}: no changelog entries, skipping`))
			continue
		}

		const products = [...parsed.entries.keys()]
		const types = [...new Set([...parsed.entries.values()].flatMap((s) => [...s.keys()]))]
		console.log(chalk.cyan(`PR #${pr.number}: ${products.join(', ')} — ${types.join(', ')}`))

		for (const [product, sections] of parsed.entries) {
			if (!targetProducts.has(product)) {
				if (product === 'app') {
					unreleasedAppPRs.push(pr.number)
				}
				continue
			}

			if (!allEntries.has(product)) {
				allEntries.set(product, [])
			}
			allEntries.get(product)!.push({ sections, prNumber: pr.number })

			const processedComment = processedComments.get(comment.id) ?? {
				body: comment.body,
				products: new Set<Product>(),
			}
			processedComment.products.add(product)
			processedComments.set(comment.id, processedComment)
		}
	}

	if (unreleasedAppPRs.length > 0) {
		console.log(
			chalk.yellow(
				`Unbaked App changelog entries found in PR(s) ${unreleasedAppPRs.join(', ')}. Run with --version to include them in an App release.`,
			),
		)
	}

	if (allEntries.size === 0) {
		console.log(chalk.yellow('No changelog entries found in merged PRs'))
		return
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
		if (args.noBake) {
			console.log(chalk.yellow('Skipping PR comment bake status updates because --no-bake was passed'))
		} else {
			for (const [commentId, { body, products }] of processedComments) {
				await markCommentProductsBaked(token, commentId, body, products)
			}
			console.log(chalk.gray(`Marked ${processedComments.size} comment(s) as baked`))
		}
	}

	console.log()
	if (args.dryRun) {
		console.log(chalk.cyan('Dry run complete — no changes written'))
	} else {
		console.log(chalk.green('Done! Review the changes in packages/blog/changelog.ts'))
	}
}

main()
