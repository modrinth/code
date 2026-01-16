import fs from 'node:fs'
import path from 'node:path'
import readline from 'node:readline'

const packageRoot = path.resolve(__dirname, '..')
const iconsDir = path.join(packageRoot, 'icons')
const lucideIconsDir = path.join(packageRoot, 'node_modules/lucide-static/icons')

function listAvailableIcons(): string[] {
	if (!fs.existsSync(lucideIconsDir)) {
		return []
	}
	return fs
		.readdirSync(lucideIconsDir)
		.filter((file) => file.endsWith('.svg'))
		.map((file) => path.basename(file, '.svg'))
		.sort()
}

function paginateList(allIcons: string[], pageSize = 20): void {
	let page = 0
	let search = ''
	let filteredIcons = allIcons

	const getFilteredIcons = (): string[] => {
		if (!search) return allIcons
		return allIcons.filter((icon) => icon.includes(search))
	}

	const renderPage = (): void => {
		console.clear()
		filteredIcons = getFilteredIcons()
		const totalPages = Math.max(1, Math.ceil(filteredIcons.length / pageSize))

		if (page >= totalPages) page = Math.max(0, totalPages - 1)

		const start = page * pageSize
		const end = Math.min(start + pageSize, filteredIcons.length)
		const pageIcons = filteredIcons.slice(start, end)

		console.log(`\x1b[1mAvailable Lucide Icons\x1b[0m`)
		console.log(`\x1b[2mSearch: \x1b[0m${search || '\x1b[2m(type to search)\x1b[0m'}\n`)

		if (pageIcons.length === 0) {
			console.log(`  \x1b[2mNo icons found matching "${search}"\x1b[0m`)
		} else {
			pageIcons.forEach((icon) => {
				if (search) {
					const highlighted = icon.replace(search, `\x1b[33m${search}\x1b[0m`)
					console.log(`  ${highlighted}`)
				} else {
					console.log(`  ${icon}`)
				}
			})
		}

		console.log(
			`\n\x1b[2m${filteredIcons.length}/${allIcons.length} icons | Page ${page + 1}/${totalPages} | ← → navigate | :q quit\x1b[0m`,
		)
	}

	renderPage()

	readline.emitKeypressEvents(process.stdin)
	if (process.stdin.isTTY) {
		process.stdin.setRawMode(true)
	}

	process.stdin.on('keypress', (str, key) => {
		if (key.ctrl && key.name === 'c') {
			console.clear()
			process.exit(0)
		}

		// :q to quit
		if (search === ':' && key.name === 'q') {
			console.clear()
			process.exit(0)
		}

		// Navigation
		if (key.name === 'right') {
			const totalPages = Math.max(1, Math.ceil(filteredIcons.length / pageSize))
			if (page < totalPages - 1) {
				page++
				renderPage()
			}
			return
		}
		if (key.name === 'left') {
			if (page > 0) {
				page--
				renderPage()
			}
			return
		}

		// Backspace
		if (key.name === 'backspace') {
			search = search.slice(0, -1)
			page = 0
			renderPage()
			return
		}

		// Escape to clear search
		if (key.name === 'escape') {
			search = ''
			page = 0
			renderPage()
			return
		}

		// Type to search
		if (str && str.length === 1 && !key.ctrl && !key.meta) {
			search += str
			page = 0
			renderPage()
		}
	})
}

function addIcon(iconId: string, overwrite: boolean): boolean {
	const sourcePath = path.join(lucideIconsDir, `${iconId}.svg`)
	const targetPath = path.join(iconsDir, `${iconId}.svg`)

	if (!fs.existsSync(sourcePath)) {
		console.error(`❌ Icon "${iconId}" not found in lucide-static`)
		console.error(`   Run with --list to see available icons`)
		return false
	}

	if (fs.existsSync(targetPath) && !overwrite) {
		console.log(`⏭️  Skipping "${iconId}" (already exists, use --overwrite to replace)`)
		return false
	}

	fs.copyFileSync(sourcePath, targetPath)
	console.log(`✅ Added "${iconId}"`)
	return true
}

function main(): void {
	const args = process.argv.slice(2)

	if (args.includes('--help') || args.includes('-h')) {
		console.log(`
Usage: pnpm icons:add [options] <icon_id> [icon_id...]

Options:
  --list, -l       Browse all available Lucide icons (interactive)
  --overwrite, -o  Overwrite existing icons
  --help, -h       Show this help message

Examples:
  pnpm icons:add heart star settings-2
  pnpm icons:add --overwrite heart
  pnpm icons:add --list              # Interactive browser
  pnpm icons:add --list | grep arrow # Pipe to grep

Interactive controls:
  Type         Search icons
  ← →          Navigate pages
  Escape       Clear search
  :q           Quit
`)
		process.exit(0)
	}

	if (args.includes('--list') || args.includes('-l')) {
		const icons = listAvailableIcons()
		if (icons.length === 0) {
			console.error('❌ lucide-static not installed. Run pnpm install first.')
			process.exit(1)
		}
		if (process.stdout.isTTY) {
			paginateList(icons)
		} else {
			// Non-interactive mode (piped output)
			icons.forEach((icon) => console.log(icon))
			process.exit(0)
		}
		return
	}

	const overwrite = args.includes('--overwrite') || args.includes('-o')
	const iconIds = args.filter((arg) => !arg.startsWith('-'))

	if (iconIds.length === 0) {
		console.error('Usage: pnpm icons:add <icon_id> [icon_id...]')
		console.error('Example: pnpm icons:add heart star settings-2')
		console.error('Run with --help for more options')
		process.exit(1)
	}

	if (!fs.existsSync(lucideIconsDir)) {
		console.error('❌ lucide-static not installed. Run pnpm install first.')
		process.exit(1)
	}

	let added = 0
	for (const iconId of iconIds) {
		if (addIcon(iconId, overwrite)) {
			added++
		}
	}

	if (added > 0) {
		console.log(`\n📦 Added ${added} icon(s). Run 'pnpm prepr:frontend:lib' to update exports.`)
	}
}

main()
