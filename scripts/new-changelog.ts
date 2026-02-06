import * as p from '@clack/prompts'
import chalk from 'chalk'
import * as fs from 'fs'
import * as path from 'path'
import { stringify } from 'yaml'

const PRODUCTS = [
	{ value: 'platform', label: 'Modrinth Platform' },
	{ value: 'app', label: 'Modrinth App' },
	{ value: 'hosting', label: 'Modrinth Hosting' },
] as const

const TYPES = [
	{ value: 'improved', label: 'Improved', hint: 'enhancement' },
	{ value: 'added', label: 'Added', hint: 'new feature' },
	{ value: 'fixed', label: 'Fixed', hint: 'bug fix' },
	{ value: 'changed', label: 'Changed', hint: 'behavioral change' },
	{ value: 'security', label: 'Security', hint: 'security fix' },
	{ value: 'removed', label: 'Removed', hint: 'removed feature' },
] as const

function slugify(text: string): string {
	return text
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, '-')
		.replace(/^-+|-+$/g, '')
		.slice(0, 50)
		.replace(/-+$/, '')
}

function cancel(): never {
	p.cancel('Cancelled.')
	process.exit(0)
}

async function main() {
	const rootDir = path.resolve(__dirname, '..')
	const changelogDir = path.join(rootDir, '.github', 'changelog')

	p.intro(chalk.cyan('Create Changelog Fragment'))

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

	const authorInput = await p.text({
		message: 'External contributor username (leave empty if Modrinth team):',
		defaultValue: '',
	})
	if (p.isCancel(authorInput)) cancel()

	const author = (authorInput as string).trim() || undefined

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

	const fragment: Record<string, unknown> = {
		product: product as string,
		type: type as string,
	}
	if (author) {
		fragment.author = author
	}
	fragment.description = descStr

	const yamlContent = stringify(fragment, { lineWidth: 80 })

	const slug = slugify(descStr)
	const filename = `${slug}.yml`
	const filepath = path.join(changelogDir, filename)

	fs.mkdirSync(changelogDir, { recursive: true })

	if (fs.existsSync(filepath)) {
		const overwrite = await p.confirm({
			message: `Fragment ${filename} already exists. Overwrite?`,
		})
		if (p.isCancel(overwrite) || !overwrite) cancel()
	}

	fs.writeFileSync(filepath, yamlContent, 'utf-8')

	p.outro(`Created ${chalk.green(`.github/changelog/${filename}`)}`)
}

main().catch((err) => {
	console.error(chalk.red('Error:'), err.message)
	process.exit(1)
})
