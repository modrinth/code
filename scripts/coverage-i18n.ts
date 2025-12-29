import { parse } from '@vue/compiler-sfc'
import * as fs from 'fs'
import * as path from 'path'

interface FileResult {
	path: string
	hasI18n: boolean
	plainStrings: string[]
	i18nUsages: number
}

interface CoverageReport {
	totalFiles: number
	filesWithI18n: number
	filesWithPlainStrings: number
	fullyConverted: number
	coverage: number
	byDirectory: Record<
		string,
		{
			total: number
			withI18n: number
			fullyConverted: number
			coverage: number
		}
	>
	filesNeedingWork: FileResult[]
}

const TRANSLATABLE_ATTRS = [
	'label',
	'placeholder',
	'title',
	'alt',
	'aria-label',
	'description',
	'header',
	'text',
	'message',
	'hint',
	'tooltip',
]

function findVueFiles(dir: string): string[] {
	const files: string[] = []

	function walk(currentDir: string) {
		const entries = fs.readdirSync(currentDir, { withFileTypes: true })
		for (const entry of entries) {
			const fullPath = path.join(currentDir, entry.name)
			if (entry.isDirectory()) {
				if (!entry.name.startsWith('.') && entry.name !== 'node_modules') {
					walk(fullPath)
				}
			} else if (entry.isFile() && entry.name.endsWith('.vue')) {
				files.push(fullPath)
			}
		}
	}

	walk(dir)
	return files
}

function isPlainTextString(text: string): boolean {
	const trimmed = text.trim()
	if (!trimmed) return false

	if (/^[\s\d\-_./\\:;,!?@#$%^&*()[\]{}|<>+=~`'"]+$/.test(trimmed)) return false

	if (/^[a-z0-9_-]+$/i.test(trimmed) && !trimmed.includes(' ')) return false

	if (trimmed.length < 2) return false

	if (/^\{\{.*\}\}$/.test(trimmed)) return false

	if (!/[a-zA-Z]/.test(trimmed)) return false

	return true
}

function extractTemplateStrings(templateContent: string): {
	plainStrings: string[]
	hasI18nPatterns: boolean
} {
	const plainStrings: string[] = []
	let hasI18nPatterns = false

	if (/formatMessage\s*\(/.test(templateContent)) {
		hasI18nPatterns = true
	}
	if (/<IntlFormatted/.test(templateContent)) {
		hasI18nPatterns = true
	}
	if (/\$t\s*\(/.test(templateContent)) {
		hasI18nPatterns = true
	}

	const tagContentRegex = />([^<]+)</g
	let match
	while ((match = tagContentRegex.exec(templateContent)) !== null) {
		const text = match[1]

		if (/^\s*\{\{.*\}\}\s*$/.test(text)) continue

		const withoutInterpolation = text.replace(/\{\{[^}]+\}\}/g, '')
		if (isPlainTextString(withoutInterpolation)) {
			plainStrings.push(text.trim())
		}
	}

	for (const attr of TRANSLATABLE_ATTRS) {
		const attrRegex = new RegExp(`(?<![:\\w])${attr}="([^"]+)"`, 'g')
		while ((match = attrRegex.exec(templateContent)) !== null) {
			const value = match[1]
			if (isPlainTextString(value)) {
				plainStrings.push(`[${attr}]: ${value}`)
			}
		}

		const singleQuoteRegex = new RegExp(`(?<![:\\w])${attr}='([^']+)'`, 'g')
		while ((match = singleQuoteRegex.exec(templateContent)) !== null) {
			const value = match[1]
			if (isPlainTextString(value)) {
				plainStrings.push(`[${attr}]: ${value}`)
			}
		}
	}

	return { plainStrings, hasI18nPatterns }
}

function checkScriptForI18n(scriptContent: string): boolean {
	const patterns = [
		/from\s+['"]@modrinth\/ui['"]/,
		/defineMessages?\s*\(/,
		/useVIntl\s*\(/,
		/formatMessage/,
		/IntlFormatted/,
		/useI18n/,
		/\$t\s*\(/,
	]

	return patterns.some((pattern) => pattern.test(scriptContent))
}

function analyzeVueFile(filePath: string): FileResult {
	const content = fs.readFileSync(filePath, 'utf-8')
	const { descriptor, errors } = parse(content)

	if (errors.length > 0) {
		console.warn(`Warning: Parse errors in ${filePath}`)
	}

	const result: FileResult = {
		path: filePath,
		hasI18n: false,
		plainStrings: [],
		i18nUsages: 0,
	}

	const scriptContent = descriptor.script?.content || descriptor.scriptSetup?.content || ''
	result.hasI18n = checkScriptForI18n(scriptContent)

	const formatMessageMatches = scriptContent.match(/formatMessage\s*\(/g)
	result.i18nUsages += formatMessageMatches?.length || 0

	if (descriptor.template?.content) {
		const templateAnalysis = extractTemplateStrings(descriptor.template.content)

		result.plainStrings = templateAnalysis.plainStrings
		if (templateAnalysis.hasI18nPatterns) {
			result.hasI18n = true
		}

		const templateFormatMessage = descriptor.template.content.match(/formatMessage\s*\(/g)
		const intlFormattedMatches = descriptor.template.content.match(/<IntlFormatted/g)
		result.i18nUsages += templateFormatMessage?.length || 0
		result.i18nUsages += intlFormattedMatches?.length || 0
	}

	return result
}

function generateReport(results: FileResult[], rootDir: string): CoverageReport {
	const report: CoverageReport = {
		totalFiles: results.length,
		filesWithI18n: 0,
		filesWithPlainStrings: 0,
		fullyConverted: 0,
		coverage: 0,
		byDirectory: {},
		filesNeedingWork: [],
	}

	for (const result of results) {
		const relativePath = path.relative(rootDir, result.path)
		const dirParts = relativePath.split(path.sep)
		const dirKey = dirParts.slice(0, 3).join('/')

		if (!report.byDirectory[dirKey]) {
			report.byDirectory[dirKey] = {
				total: 0,
				withI18n: 0,
				fullyConverted: 0,
				coverage: 0,
			}
		}

		report.byDirectory[dirKey].total++

		if (result.hasI18n) {
			report.filesWithI18n++
			report.byDirectory[dirKey].withI18n++
		}

		if (result.plainStrings.length > 0) {
			report.filesWithPlainStrings++
			report.filesNeedingWork.push(result)
		} else if (result.hasI18n || result.i18nUsages > 0) {
			report.fullyConverted++
			report.byDirectory[dirKey].fullyConverted++
		}
	}

	report.coverage =
		report.totalFiles > 0 ? Math.round((report.fullyConverted / report.totalFiles) * 100) : 0

	for (const dir of Object.keys(report.byDirectory)) {
		const dirStats = report.byDirectory[dir]
		dirStats.coverage =
			dirStats.total > 0 ? Math.round((dirStats.fullyConverted / dirStats.total) * 100) : 0
	}

	return report
}

function printReport(report: CoverageReport, rootDir: string, verbose: boolean) {
	console.log('\n' + '='.repeat(60))
	console.log('i18n Coverage Report')
	console.log('='.repeat(60))

	console.log(`\nTotal Vue files:        ${report.totalFiles}`)
	console.log(`Files using i18n:       ${report.filesWithI18n}`)
	console.log(`Fully converted:        ${report.fullyConverted}`)
	console.log(`Need work:              ${report.filesWithPlainStrings}`)
	console.log(`\nOverall coverage:       ${report.coverage}%`)

	console.log('\n' + '-'.repeat(60))
	console.log('Coverage by directory:')
	console.log('-'.repeat(60))

	const sortedDirs = Object.entries(report.byDirectory).sort(([, a], [, b]) => b.total - a.total)

	for (const [dir, stats] of sortedDirs) {
		const bar =
			'█'.repeat(Math.floor(stats.coverage / 5)) + '░'.repeat(20 - Math.floor(stats.coverage / 5))
		console.log(
			`${dir.padEnd(40)} ${stats.coverage.toString().padStart(3)}% ${bar} (${stats.fullyConverted}/${stats.total})`,
		)
	}

	if (verbose && report.filesNeedingWork.length > 0) {
		console.log('\n' + '-'.repeat(60))
		console.log('Files needing i18n work:')
		console.log('-'.repeat(60))

		const sorted = [...report.filesNeedingWork].sort(
			(a, b) => b.plainStrings.length - a.plainStrings.length,
		)

		for (const file of sorted.slice(0, 50)) {
			const relativePath = path.relative(rootDir, file.path)
			console.log(`\n${relativePath} (${file.plainStrings.length} plain strings)`)
			if (verbose) {
				for (const str of file.plainStrings.slice(0, 5)) {
					const truncated = str.length > 60 ? str.slice(0, 57) + '...' : str
					console.log(`  - ${truncated}`)
				}
				if (file.plainStrings.length > 5) {
					console.log(`  ... and ${file.plainStrings.length - 5} more`)
				}
			}
		}

		if (sorted.length > 50) {
			console.log(`\n... and ${sorted.length - 50} more files`)
		}
	}

	console.log('\n' + '='.repeat(60))
}

function main() {
	const args = process.argv.slice(2)
	const verbose = args.includes('--verbose') || args.includes('-v')
	const jsonOutput = args.includes('--json')

	const rootDir = path.resolve(__dirname, '..')
	const frontendDir = path.join(rootDir, 'apps/frontend/src')
	const appFrontendDir = path.join(rootDir, 'apps/app-frontend/src')

	console.log('Scanning Vue files...')

	const allFiles: string[] = []

	if (fs.existsSync(frontendDir)) {
		allFiles.push(...findVueFiles(frontendDir))
	}

	if (fs.existsSync(appFrontendDir)) {
		allFiles.push(...findVueFiles(appFrontendDir))
	}

	console.log(`Found ${allFiles.length} Vue files`)

	const results: FileResult[] = []
	for (const file of allFiles) {
		try {
			results.push(analyzeVueFile(file))
		} catch (err) {
			console.warn(`Error analyzing ${file}:`, err)
		}
	}

	const report = generateReport(results, rootDir)

	if (jsonOutput) {
		console.log(JSON.stringify(report, null, 2))
	} else {
		printReport(report, rootDir, verbose)
	}
}

main()
