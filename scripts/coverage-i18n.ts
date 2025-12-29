import { parse } from '@vue/compiler-sfc'
import chalk from 'chalk'
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

const theme = {
	primary: chalk.cyan,
	success: chalk.green,
	warning: chalk.yellow,
	error: chalk.red,
	muted: chalk.gray,
	highlight: chalk.white.bold,
	title: chalk.bold.cyan,
	subtitle: chalk.dim,
}

const icons = {
	check: chalk.green('✓'),
	cross: chalk.red('✗'),
	arrow: chalk.cyan('→'),
	dot: '●',
	warning: chalk.yellow('⚠'),
	file: '◦',
	folder: '▸',
	globe: '◎',
	sparkle: chalk.yellow('★'),
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
				if (!entry.name.startsWith('.') && entry.name !== 'node_modules' && entry.name !== 'legal') {
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

	if (/formatMessage\s*\(/.test(templateContent)) hasI18nPatterns = true
	if (/<IntlFormatted/.test(templateContent)) hasI18nPatterns = true
	if (/\$t\s*\(/.test(templateContent)) hasI18nPatterns = true

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
			if (isPlainTextString(match[1])) {
				plainStrings.push(`[${attr}]: ${match[1]}`)
			}
		}
		const singleQuoteRegex = new RegExp(`(?<![:\\w])${attr}='([^']+)'`, 'g')
		while ((match = singleQuoteRegex.exec(templateContent)) !== null) {
			if (isPlainTextString(match[1])) {
				plainStrings.push(`[${attr}]: ${match[1]}`)
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
	const { descriptor } = parse(content)

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
			report.byDirectory[dirKey] = { total: 0, withI18n: 0, fullyConverted: 0, coverage: 0 }
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

function progressBar(percent: number, width: number = 20): string {
	const filled = Math.round((percent / 100) * width)
	const empty = width - filled

	let color: (s: string) => string
	if (percent >= 80) color = chalk.green
	else if (percent >= 50) color = chalk.yellow
	else if (percent >= 25) color = chalk.hex('#FFA500')
	else color = chalk.red

	return color('━'.repeat(filled)) + chalk.gray('━'.repeat(empty))
}

function colorPercent(percent: number): string {
	if (percent >= 80) return chalk.green.bold(`${percent}%`)
	if (percent >= 50) return chalk.yellow.bold(`${percent}%`)
	if (percent >= 25) return chalk.hex('#FFA500').bold(`${percent}%`)
	return chalk.red.bold(`${percent}%`)
}

function printReport(report: CoverageReport, rootDir: string, verbose: boolean) {
	console.log()
	console.log(theme.title(`  ${icons.globe} i18n Coverage Report`))
	console.log(theme.muted(`  ${'─'.repeat(45)}`))
	console.log()

	console.log(chalk.bold('  Summary'))
	console.log()
	console.log(`    ${theme.muted('Total files')}      ${theme.highlight(report.totalFiles)}`)
	console.log(`    ${theme.muted('Using i18n')}       ${theme.highlight(report.filesWithI18n)}`)
	console.log(
		`    ${theme.muted('Converted')}        ${report.fullyConverted > 0 ? chalk.green.bold(report.fullyConverted) : theme.highlight(report.fullyConverted)}`,
	)
	console.log(
		`    ${theme.muted('Need work')}        ${report.filesWithPlainStrings > 0 ? chalk.yellow.bold(report.filesWithPlainStrings) : theme.highlight(report.filesWithPlainStrings)}`,
	)
	console.log()
	console.log(`    ${theme.muted('Coverage')}         ${colorPercent(report.coverage)}`)
	console.log(`    ${progressBar(report.coverage, 32)}`)
	console.log()

	console.log(theme.muted(`  ${'─'.repeat(45)}`))
	console.log(chalk.bold('  By Directory'))
	console.log()

	const sortedDirs = Object.entries(report.byDirectory).sort(([, a], [, b]) => b.total - a.total)

	for (const [dir, stats] of sortedDirs) {
		const shortDir = dir.replace('apps/', '').replace('/src', '')
		const paddedDir = shortDir.padEnd(20)

		console.log(
			`    ${theme.primary(paddedDir)} ${colorPercent(stats.coverage).padStart(12)} ${progressBar(stats.coverage, 12)} ${theme.muted(`${stats.fullyConverted}/${stats.total}`)}`,
		)
	}
	console.log()

	if (verbose && report.filesNeedingWork.length > 0) {
		console.log(theme.muted(`  ${'─'.repeat(45)}`))
		console.log(chalk.bold('  Files Needing Work'))
		console.log()

		const sorted = [...report.filesNeedingWork].sort(
			(a, b) => b.plainStrings.length - a.plainStrings.length,
		)

		for (const file of sorted.slice(0, 20)) {
			const relativePath = path.relative(rootDir, file.path)
			const shortPath = relativePath.replace('apps/', '').replace('/src/', '/')
			const count = file.plainStrings.length

			let countStr: string
			if (count >= 50) countStr = chalk.red.bold(`${count}`)
			else if (count >= 20) countStr = chalk.yellow.bold(`${count}`)
			else countStr = chalk.white(`${count}`)

			console.log(`    ${icons.arrow} ${chalk.white(shortPath)}`)
			console.log(`      ${countStr} ${theme.muted('plain strings')}`)

			for (const str of file.plainStrings.slice(0, 2)) {
				const cleaned = str.replace(/\n/g, ' ').replace(/\t/g, ' ').trim()
				const truncated = cleaned.length > 45 ? cleaned.slice(0, 42) + '...' : cleaned
				console.log(`      ${theme.muted(`"${truncated}"`)}`)
			}

			if (file.plainStrings.length > 2) {
				console.log(`      ${theme.subtitle(`+${file.plainStrings.length - 2} more`)}`)
			}
			console.log()
		}

		if (sorted.length > 20) {
			console.log(`    ${theme.subtitle(`... and ${sorted.length - 20} more files`)}`)
			console.log()
		}
	}

	console.log(theme.muted(`  ${'─'.repeat(45)}`))
	if (!verbose) {
		console.log(theme.subtitle(`  Run with ${chalk.cyan('--verbose')} to see files needing work`))
	}
	console.log()
}

function main() {
	const args = process.argv.slice(2)
	const verbose = args.includes('--verbose') || args.includes('-v')
	const jsonOutput = args.includes('--json')

	const rootDir = path.resolve(__dirname, '..')
	const frontendDir = path.join(rootDir, 'apps/frontend/src')
	const appFrontendDir = path.join(rootDir, 'apps/app-frontend/src')

	if (!jsonOutput) {
		console.log()
		process.stdout.write(theme.muted('  Scanning Vue files... '))
	}

	const allFiles: string[] = []

	if (fs.existsSync(frontendDir)) {
		allFiles.push(...findVueFiles(frontendDir))
	}

	if (fs.existsSync(appFrontendDir)) {
		allFiles.push(...findVueFiles(appFrontendDir))
	}

	if (!jsonOutput) {
		console.log(`${icons.check} ${theme.highlight(allFiles.length)} files`)
	}

	const results: FileResult[] = []
	for (const file of allFiles) {
		try {
			results.push(analyzeVueFile(file))
		} catch {
			// Silent fail
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
