import { readFileSync, writeFileSync } from 'node:fs'
import { globSync } from 'glob'

const APPS_DIR = new URL('../apps', import.meta.url).pathname

// Patterns to match old imports
const I18N_VINTL_PATTERN = /^import\s+(?:type\s+)?(\{[^}]+\})\s+from\s+['"](?:~|@)\/utils\/i18n-vintl['"]/gm
const INTL_FORMATTED_PATTERN = /^import\s+(\w+)\s+from\s+['"](?:~|@)\/components\/ui\/IntlFormatted\.vue['"]/gm
const MODRINTH_UI_PATTERN = /^(import\s+(?:type\s+)?)(\{[^}]+\})(\s+from\s+['"]@modrinth\/ui['"])/m

interface ImportInfo {
	names: string[]
	isTypeOnly: boolean
}

function extractImportNames(braceContent: string): string[] {
	// Remove braces and split by comma
	const content = braceContent.replace(/^\{|\}$/g, '').trim()
	return content
		.split(',')
		.map((s) => s.trim())
		.filter((s) => s.length > 0)
}

function processFile(filePath: string): boolean {
	let content = readFileSync(filePath, 'utf-8')
	const originalContent = content
	const newImports: ImportInfo = { names: [], isTypeOnly: false }

	// Find and remove i18n-vintl imports
	content = content.replace(I18N_VINTL_PATTERN, (match, braceContent) => {
		const isTypeImport = match.includes('import type')
		const names = extractImportNames(braceContent)
		newImports.names.push(...names)
		if (isTypeImport && newImports.names.length === names.length) {
			newImports.isTypeOnly = true
		} else if (!isTypeImport) {
			newImports.isTypeOnly = false
		}
		return '' // Remove the line
	})

	// Find and remove IntlFormatted imports
	content = content.replace(INTL_FORMATTED_PATTERN, (_match, name) => {
		newImports.names.push(name)
		newImports.isTypeOnly = false // Component import is never type-only
		return '' // Remove the line
	})

	// If no imports were found, nothing to do
	if (newImports.names.length === 0) {
		return false
	}

	// Clean up empty lines left by removed imports
	content = content.replace(/\n\n\n+/g, '\n\n')

	// Check if there's already a @modrinth/ui import
	const existingMatch = content.match(MODRINTH_UI_PATTERN)

	if (existingMatch) {
		// Merge into existing import
		const [fullMatch, prefix, existingBraces, suffix] = existingMatch
		const existingNames = extractImportNames(existingBraces)
		const allNames = [...new Set([...existingNames, ...newImports.names])]
		const newBraces = `{ ${allNames.join(', ')} }`
		content = content.replace(fullMatch, `${prefix}${newBraces}${suffix}`)
	} else {
		// Add new import after the last import statement
		const importKeyword = newImports.isTypeOnly ? 'import type' : 'import'
		const newImportLine = `${importKeyword} { ${newImports.names.join(', ')} } from '@modrinth/ui'`

		// Find the last import statement and insert after it
		const lines = content.split('\n')
		let lastImportIndex = -1
		for (let i = 0; i < lines.length; i++) {
			if (lines[i].startsWith('import ') || lines[i].startsWith('import\t')) {
				lastImportIndex = i
			}
		}

		if (lastImportIndex >= 0) {
			lines.splice(lastImportIndex + 1, 0, newImportLine)
			content = lines.join('\n')
		} else {
			// No imports found, add at the top (after script tag if .vue)
			if (filePath.endsWith('.vue')) {
				content = content.replace(/(<script[^>]*>)/, `$1\n${newImportLine}`)
			} else {
				content = newImportLine + '\n' + content
			}
		}
	}

	if (content !== originalContent) {
		writeFileSync(filePath, content)
		return true
	}
	return false
}

function main() {
	const files = globSync(`${APPS_DIR}/**/*.{vue,ts}`, {
		ignore: ['**/node_modules/**', '**/dist/**', '**/.nuxt/**'],
	})

	console.log(`Found ${files.length} files to process`)

	let modified = 0
	for (const file of files) {
		if (processFile(file)) {
			console.log(`  Modified: ${file.replace(APPS_DIR, 'apps')}`)
			modified++
		}
	}

	console.log(`\nDone! Modified ${modified} files.`)
}

main()
