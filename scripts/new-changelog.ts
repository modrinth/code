import * as p from '@clack/prompts'
import chalk from 'chalk'
import * as crypto from 'crypto'
import * as fs from 'fs'
import * as path from 'path'
import { stringify } from 'yaml'

const PRODUCTS = [
	{ value: 'platform', label: 'Modrinth Platform' },
	{ value: 'app', label: 'Modrinth App' },
	{ value: 'hosting', label: 'Modrinth Hosting' },
] as const

const VALID_PRODUCTS = PRODUCTS.map((x) => x.value)

const TYPES = [
	{ value: 'improved', label: 'Improved', hint: 'enhancement' },
	{ value: 'added', label: 'Added', hint: 'new feature' },
	{ value: 'fixed', label: 'Fixed', hint: 'bug fix' },
	{ value: 'changed', label: 'Changed', hint: 'behavioral change' },
	{ value: 'security', label: 'Security', hint: 'security fix' },
	{ value: 'removed', label: 'Removed', hint: 'removed feature' },
] as const

const TYPE_PREFIXES: Record<string, string> = {
	fixed: 'fixed',
	fix: 'fixed',
	added: 'added',
	add: 'added',
	improved: 'improved',
	improve: 'improved',
	changed: 'changed',
	change: 'changed',
	removed: 'removed',
	remove: 'removed',
	security: 'security',
}

function hashContent(text: string): string {
	return crypto.createHash('sha256').update(text).digest('hex').slice(0, 8)
}

function inferType(description: string): string {
	const firstWord = description.split(/\s/)[0].toLowerCase().replace(/[^a-z]/g, '')
	return TYPE_PREFIXES[firstWord] ?? 'improved'
}

function writeFragment(
	changelogDir: string,
	product: string,
	type: string,
	description: string,
	author?: string,
): string {
	const fragment: Record<string, unknown> = {
		product,
		type,
	}
	if (author) {
		fragment.author = author
	}
	fragment.description = description

	const yamlContent = stringify(fragment, { lineWidth: 80 })

	const hash = hashContent(yamlContent)
	const filename = `${product}-${type}-${hash}.yml`
	const filepath = path.join(changelogDir, filename)

	fs.mkdirSync(changelogDir, { recursive: true })
	fs.writeFileSync(filepath, yamlContent, 'utf-8')

	return filename
}

function cancel(): never {
	p.cancel('Cancelled.')
	process.exit(0)
}

function printUsage(): void {
	console.log(`Usage:
  pnpm changelog <product> "description"    Fast path (type inferred from prefix)
  pnpm changelog <product> "description" --author <name>
  pnpm changelog                            Interactive mode

Products: ${VALID_PRODUCTS.join(', ')}

Type is inferred from description prefix:
  "Fixed ..."    → fixed       "Added ..."     → added
  "Improved ..." → improved    "Changed ..."   → changed
  "Removed ..."  → removed     "Security ..."  → security
  No match       → improved (default)`)
}

function parseFastArgs(argv: string[]): {
	product: string
	description: string
	author?: string
} | null {
	const positional: string[] = []
	let author: string | undefined

	let i = 0
	while (i < argv.length) {
		if (argv[i] === '--author') {
			i++
			if (i >= argv.length) {
				console.error(chalk.red('--author requires a value'))
				process.exit(1)
			}
			author = argv[i]
			i++
		} else if (argv[i] === '--help' || argv[i] === '-h') {
			printUsage()
			process.exit(0)
		} else if (argv[i].startsWith('--')) {
			console.error(chalk.red(`Unknown flag: ${argv[i]}`))
			printUsage()
			process.exit(1)
		} else {
			positional.push(argv[i])
			i++
		}
	}

	if (positional.length === 0) return null
	if (positional.length < 2) {
		console.error(chalk.red('Fast path requires: <product> "description"'))
		printUsage()
		process.exit(1)
	}

	const product = positional[0]
	if (!VALID_PRODUCTS.includes(product)) {
		console.error(chalk.red(`Unknown product: ${product}`))
		console.error(`Valid products: ${VALID_PRODUCTS.join(', ')}`)
		process.exit(1)
	}

	const description = positional.slice(1).join(' ').trim()
	if (!description) {
		console.error(chalk.red('Description is required'))
		process.exit(1)
	}

	return { product, description, author }
}

async function interactive(changelogDir: string) {
	p.intro(chalk.cyan('Create Changelog Fragment'))

	const createdFiles: string[] = []

	while (true) {
		const product = await p.select({
			message: 'Which product is this change for?',
			options: PRODUCTS.map((x) => ({ value: x.value, label: x.label })),
		})
		if (p.isCancel(product)) cancel()

		const type = await p.select({
			message: 'What type of change is this?',
			options: TYPES.map((x) => ({ value: x.value, label: x.label, hint: x.hint })),
		})
		if (p.isCancel(type)) cancel()

		const description = await p.text({
			message: 'Description of the change:',
			validate: (val) => {
				if (!val.trim()) return 'Description is required'
				return undefined
			},
		})
		if (p.isCancel(description)) cancel()

		const descStr = (description as string).trim()

		if (descStr.includes('`')) {
			p.log.warn('Description contains backticks — these may cause issues in the baked changelog.')
		}

		const filename = writeFragment(changelogDir, product as string, type as string, descStr)
		createdFiles.push(filename)

		p.log.success(`Created ${chalk.green(`.github/changelog/${filename}`)}`)

		const another = await p.confirm({
			message: 'Add another changelog entry?',
		})
		if (p.isCancel(another) || !another) break
	}

	p.outro(`Created ${createdFiles.length} fragment(s)`)
}

async function main() {
	const rootDir = path.resolve(__dirname, '..')
	const changelogDir = path.join(rootDir, '.github', 'changelog')

	const args = process.argv.slice(2)
	const fast = parseFastArgs(args)

	if (fast) {
		const { product, description, author } = fast
		const type = inferType(description)

		if (description.includes('`')) {
			console.warn(
				chalk.yellow('Warning: Description contains backticks — these may cause issues in the baked changelog.'),
			)
		}

		const filename = writeFragment(changelogDir, product, type, description, author)
		console.log(chalk.green(`Created .github/changelog/${filename}`))
	} else {
		await interactive(changelogDir)
	}
}

main().catch((err) => {
	console.error(chalk.red('Error:'), err.message)
	process.exit(1)
})
