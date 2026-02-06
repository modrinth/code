import { execSync } from 'child_process'
import chalk from 'chalk'
import * as fs from 'fs'
import * as path from 'path'
import { parse } from 'yaml'

interface ChangelogFragment {
	product: 'platform' | 'app' | 'hosting'
	type: 'added' | 'improved' | 'fixed' | 'security' | 'changed' | 'removed'
	pr?: number
	author?: string
	description: string
}

const PRODUCT_MAP: Record<string, string> = {
	platform: 'web',
	app: 'app',
	hosting: 'hosting',
}

const VALID_PRODUCTS = ['platform', 'app', 'hosting']
const PRODUCT_ALIASES: Record<string, string[]> = {
	web: ['platform', 'hosting'],
}
const VALID_TYPES = ['added', 'improved', 'fixed', 'security', 'changed', 'removed']

const TYPE_TO_HEADER: Record<string, string> = {
	security: '## Security fixes',
	added: '## Added',
	improved: '## Improvements',
	fixed: '## Improvements',
	removed: '## Improvements',
	changed: '## Changes',
}

const SECTION_ORDER = ['## Security fixes', '## Added', '## Improvements', '## Changes']

const IMPROVEMENTS_SUB_ORDER = ['improved', 'fixed', 'removed']

function parseArgs(argv: string[]): {
	products: string[]
	version?: string
	extract?: boolean
} {
	const products: string[] = []
	let version: string | undefined
	let extract = false

	let i = 0
	while (i < argv.length) {
		if (argv[i] === '--') {
			i++
			continue
		}
		if (argv[i] === '--product') {
			i++
			while (i < argv.length && !argv[i].startsWith('--')) {
				const p = argv[i]
				if (PRODUCT_ALIASES[p]) {
					for (const alias of PRODUCT_ALIASES[p]) {
						if (!products.includes(alias)) products.push(alias)
					}
				} else if (VALID_PRODUCTS.includes(p)) {
					if (!products.includes(p)) products.push(p)
				} else {
					console.error(chalk.red(`Unknown product: ${p}`))
					console.error(
						`Valid products: ${VALID_PRODUCTS.join(', ')} (aliases: ${Object.keys(PRODUCT_ALIASES).join(', ')})`,
					)
					process.exit(1)
				}
				i++
			}
		} else if (argv[i] === '--version') {
			i++
			if (i >= argv.length || argv[i].startsWith('--')) {
				console.error(chalk.red('--version requires a value'))
				process.exit(1)
			}
			version = argv[i]
			i++
		} else if (argv[i] === '--extract') {
			extract = true
			i++
		} else {
			console.error(chalk.red(`Unknown argument: ${argv[i]}`))
			process.exit(1)
		}
	}

	if (products.length === 0) {
		console.error(chalk.red('At least one --product is required'))
		console.error(
			'Usage: pnpm scripts bake-changelog -- --product <platform|app|hosting|web> [--version X.Y.Z] [--extract]',
		)
		process.exit(1)
	}

	if (extract && products.length !== 1) {
		console.error(chalk.red('--extract requires exactly one --product'))
		process.exit(1)
	}

	if (products.includes('app') && !version) {
		console.error(chalk.red('--version is required when baking app changelog'))
		process.exit(1)
	}

	if (!products.includes('app') && version) {
		console.error(chalk.yellow('Warning: --version is only used for app product, ignoring'))
		version = undefined
	}

	return { products, version, extract }
}

function readFragments(
	changelogDir: string,
): { fragment: ChangelogFragment; filename: string }[] {
	if (!fs.existsSync(changelogDir)) {
		return []
	}

	const files = fs
		.readdirSync(changelogDir)
		.filter((f) => f.endsWith('.yml') || f.endsWith('.yaml'))
		.sort()

	const fragments: { fragment: ChangelogFragment; filename: string }[] = []

	for (const filename of files) {
		const filepath = path.join(changelogDir, filename)
		const content = fs.readFileSync(filepath, 'utf-8')

		try {
			const parsed = parse(content) as ChangelogFragment

			if (!parsed.product || !parsed.type || !parsed.description) {
				console.error(chalk.yellow(`Warning: Skipping ${filename} — missing required fields`))
				continue
			}

			if (!VALID_PRODUCTS.includes(parsed.product)) {
				console.error(
					chalk.yellow(`Warning: Skipping ${filename} — invalid product: ${parsed.product}`),
				)
				continue
			}

			if (!VALID_TYPES.includes(parsed.type)) {
				console.error(
					chalk.yellow(`Warning: Skipping ${filename} — invalid type: ${parsed.type}`),
				)
				continue
			}

			fragments.push({ fragment: parsed, filename })
		} catch (err) {
			console.error(
				chalk.yellow(
					`Warning: Skipping ${filename} — invalid YAML: ${(err as Error).message}`,
				),
			)
		}
	}

	return fragments
}

function resolvePr(changelogDir: string, filename: string): number | undefined {
	try {
		const filepath = path.join(changelogDir, filename)
		const subject = execSync(`git log --diff-filter=A --format=%s -1 -- "${filepath}"`, {
			encoding: 'utf-8',
		}).trim()

		if (!subject) return undefined

		// Squash merge: "Some change (#1234)"
		const squashMatch = subject.match(/\(#(\d+)\)\s*$/)
		if (squashMatch) return parseInt(squashMatch[1], 10)

		// Merge commit: "Merge pull request #1234 from ..."
		const mergeMatch = subject.match(/Merge pull request #(\d+)/)
		if (mergeMatch) return parseInt(mergeMatch[1], 10)

		return undefined
	} catch {
		return undefined
	}
}

function formatLine(fragment: ChangelogFragment): string {
	const desc = fragment.description.trim()
	if (fragment.author && fragment.pr) {
		return `- ${desc} ([#${fragment.pr}](https://github.com/modrinth/code/pull/${fragment.pr}) by @${fragment.author})`
	}
	return `- ${desc}`
}

function buildBody(fragments: ChangelogFragment[]): string {
	const sections = new Map<string, ChangelogFragment[]>()

	for (const fragment of fragments) {
		const header = TYPE_TO_HEADER[fragment.type]
		if (!sections.has(header)) {
			sections.set(header, [])
		}
		sections.get(header)!.push(fragment)
	}

	const parts: string[] = []

	for (const header of SECTION_ORDER) {
		const sectionFragments = sections.get(header)
		if (!sectionFragments || sectionFragments.length === 0) continue

		if (header === '## Improvements') {
			sectionFragments.sort(
				(a, b) => IMPROVEMENTS_SUB_ORDER.indexOf(a.type) - IMPROVEMENTS_SUB_ORDER.indexOf(b.type),
			)
		}

		const lines = sectionFragments.map(formatLine)
		parts.push(`${header}\n${lines.join('\n')}`)
	}

	return parts.join('\n\n')
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

	// Determine PST vs PDT by comparing UTC offset
	const jan = new Date(now.getFullYear(), 0, 1)
	const jul = new Date(now.getFullYear(), 6, 1)
	const stdOffset = Math.max(jan.getTimezoneOffset(), jul.getTimezoneOffset())
	// Create a date in Pacific time to check if DST is active
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
	const tsProduct = PRODUCT_MAP[product]
	const versionLine = version ? `\n\t\tversion: '${version}',` : ''

	return `\t{
\t\tdate: \`${dateStr}\`,
\t\tproduct: '${tsProduct}',${versionLine}
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

function deleteFragments(changelogDir: string, filenames: string[]): void {
	for (const filename of filenames) {
		fs.unlinkSync(path.join(changelogDir, filename))
	}
}

function extractFromChangelog(changelogPath: string, product: string, version?: string): void {
	const content = fs.readFileSync(changelogPath, 'utf-8')
	const tsProduct = PRODUCT_MAP[product]

	// Match entries in the VERSIONS array by finding the product and optional version
	// The entries look like:
	//   product: 'app',
	//   version: '0.10.28',
	//   body: `...`,
	const entryRegex = version
		? new RegExp(
				`product:\\s*'${tsProduct}',\\s*\\n\\s*version:\\s*'${version.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}',\\s*\\n\\s*body:\\s*\`([\\s\\S]*?)\`,`,
			)
		: new RegExp(`product:\\s*'${tsProduct}',\\s*\\n\\s*body:\\s*\`([\\s\\S]*?)\`,`)

	const match = content.match(entryRegex)

	if (!match) {
		console.error(
			chalk.red(
				`Could not find a ${tsProduct}${version ? ` v${version}` : ''} entry in changelog.ts`,
			),
		)
		process.exit(1)
	}

	// Output just the body to stdout
	process.stdout.write(match[1])
}

function main() {
	const args = process.argv.slice(2)
	const { products, version, extract } = parseArgs(args)

	const rootDir = path.resolve(__dirname, '..')
	const changelogDir = path.join(rootDir, '.github', 'changelog')
	const changelogPath = path.join(rootDir, 'packages', 'blog', 'changelog.ts')

	if (extract) {
		extractFromChangelog(changelogPath, products[0], version)
		return
	}

	const allFragments = readFragments(changelogDir)

	// Resolve PR numbers from git history for fragments that don't have one
	for (const { fragment, filename } of allFragments) {
		if (!fragment.pr) {
			const pr = resolvePr(changelogDir, filename)
			if (pr) {
				fragment.pr = pr
			} else {
				console.log(
					chalk.gray(`Could not resolve PR for ${filename} — no link will be added`),
				)
			}
		}
	}

	if (allFragments.length === 0) {
		console.error(chalk.yellow('No changelog fragments found in .github/changelog/'))
		process.exit(1)
	}

	// Process in reverse so the last product processed ends up at the top of the array
	const productsToProcess = [...products].reverse()

	for (const product of productsToProcess) {
		const productFragments = allFragments.filter((f) => f.fragment.product === product)

		if (productFragments.length === 0) {
			console.log(chalk.yellow(`No fragments found for product: ${product}`))
			continue
		}

		const fragments = productFragments.map((f) => f.fragment)
		const filenames = productFragments.map((f) => f.filename)

		const body = buildBody(fragments)

		if (body.includes('`')) {
			console.error(
				chalk.yellow(
					`Warning: Baked body for ${product} contains backticks — this may break the template literal in changelog.ts`,
				),
			)
		}

		const entryVersion = product === 'app' ? version : undefined
		const entryString = generateEntry(product, body, entryVersion)

		insertIntoChangelog(changelogPath, entryString)

		console.log(chalk.green(`Baked ${productFragments.length} fragment(s) for ${product}`))

		if (product === 'app') {
			console.log()
			console.log(chalk.cyan('GitHub release body:'))
			console.log(chalk.cyan('---'))
			console.log(body)
			console.log(chalk.cyan('---'))
		}

		deleteFragments(changelogDir, filenames)
		console.log(chalk.gray(`Deleted ${filenames.length} fragment file(s)`))
	}

	console.log()
	console.log(chalk.green('Done! Review the changes in packages/blog/changelog.ts'))
}

main()
