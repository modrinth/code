import { parse as parseVue } from '@vue/compiler-sfc'
import {
	parse as parseTemplate,
	NodeTypes,
	type RootNode,
	type TemplateChildNode,
	type ElementNode,
	type AttributeNode,
	type TextNode,
} from '@vue/compiler-dom'
import { parse as parseTs, AST_NODE_TYPES } from '@typescript-eslint/typescript-estree'
import type { TSESTree } from '@typescript-eslint/typescript-estree'
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

const TRANSLATABLE_ATTRS = new Set([
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
])

// i18n symbols that indicate i18n usage
const I18N_SYMBOLS = ['useVIntl', 'defineMessage', 'defineMessages', 'IntlFormatted', 'useI18n'] as const
const I18N_CALL_PATTERNS = ['formatMessage', '$t'] as const

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
	if (trimmed.length < 2) return false
	// Only punctuation/symbols/numbers
	if (/^[\s\d\-_./\\:;,!?@#$%^&*()[\]{}|<>+=~`'"]+$/.test(trimmed)) return false
	// Single identifier-like word (no spaces)
	if (/^[a-z0-9_-]+$/i.test(trimmed) && !trimmed.includes(' ')) return false
	// Just a Vue interpolation
	if (/^\{\{.*\}\}$/.test(trimmed)) return false
	// No letters at all
	if (!/[a-zA-Z]/.test(trimmed)) return false
	// URLs
	if (/^https?:\/\//.test(trimmed)) return false
	// File/route paths (but not "/ month" style text)
	if (/^\/[a-zA-Z_][\w\-/[\]]*$/.test(trimmed)) return false
	// Email addresses
	if (/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(trimmed)) return false
	return true
}

/**
 * Walk TypeScript AST and call visitor for each node
 */
function walkTsAst(node: TSESTree.Node, visitor: (node: TSESTree.Node) => void) {
	visitor(node)

	for (const key of Object.keys(node)) {
		const child = (node as unknown as Record<string, unknown>)[key]
		if (child && typeof child === 'object') {
			if (Array.isArray(child)) {
				for (const item of child) {
					if (item && typeof item === 'object' && 'type' in item) {
						walkTsAst(item as TSESTree.Node, visitor)
					}
				}
			} else if ('type' in child) {
				walkTsAst(child as TSESTree.Node, visitor)
			}
		}
	}
}

/**
 * Walk Vue template AST and call visitor for each node
 */
function walkTemplateAst(
	node: RootNode | TemplateChildNode,
	visitor: (node: RootNode | TemplateChildNode) => void,
) {
	visitor(node)

	if ('children' in node && Array.isArray(node.children)) {
		for (const child of node.children as TemplateChildNode[]) {
			walkTemplateAst(child, visitor)
		}
	}

	// Handle v-if/v-for branches
	if (node.type === NodeTypes.IF) {
		for (const branch of node.branches) {
			walkTemplateAst(branch, visitor)
		}
	}

	if (node.type === NodeTypes.FOR) {
		for (const child of node.children) {
			walkTemplateAst(child, visitor)
		}
	}
}

/**
 * Parse TypeScript/JavaScript content into AST
 */
function parseTsContent(content: string, isJsx: boolean = false): TSESTree.Program | null {
	try {
		return parseTs(content, {
			jsx: isJsx,
			loc: true,
			range: true,
		})
	} catch {
		return null
	}
}

/**
 * Count i18n calls in a JavaScript expression using AST
 */
function countI18nCallsInExpression(expression: string): number {
	// Wrap expression to make it parseable
	const wrappedCode = `(${expression})`
	const ast = parseTsContent(wrappedCode, false)
	if (!ast) return 0

	let count = 0
	walkTsAst(ast, (node) => {
		if (node.type === AST_NODE_TYPES.CallExpression) {
			const callee = node.callee
			if (callee.type === AST_NODE_TYPES.Identifier) {
				if (I18N_CALL_PATTERNS.includes(callee.name as (typeof I18N_CALL_PATTERNS)[number])) {
					count++
				}
			}
			// Also handle this.formatMessage() or intl.formatMessage()
			if (callee.type === AST_NODE_TYPES.MemberExpression && callee.property.type === AST_NODE_TYPES.Identifier) {
				if (I18N_CALL_PATTERNS.includes(callee.property.name as (typeof I18N_CALL_PATTERNS)[number])) {
					count++
				}
			}
		}
	})
	return count
}

/**
 * Check if script has i18n imports or usage using AST
 */
function checkScriptForI18n(scriptContent: string): { hasI18n: boolean; i18nUsages: number } {
	const ast = parseTsContent(scriptContent, true)
	if (!ast) {
		return { hasI18n: false, i18nUsages: 0 }
	}

	let hasI18n = false
	let i18nUsages = 0

	// Check imports
	for (const node of ast.body) {
		if (node.type === AST_NODE_TYPES.ImportDeclaration) {
			const source = node.source.value as string
			// Check for @modrinth/ui import
			if (source === '@modrinth/ui') {
				for (const specifier of node.specifiers) {
					if (specifier.type === AST_NODE_TYPES.ImportSpecifier) {
						const importedName =
							specifier.imported.type === AST_NODE_TYPES.Identifier
								? specifier.imported.name
								: String(specifier.imported.value)
						if (I18N_SYMBOLS.includes(importedName as (typeof I18N_SYMBOLS)[number])) {
							hasI18n = true
						}
					}
				}
			}
		}
	}

	// Walk AST for call expressions
	walkTsAst(ast, (node) => {
		if (node.type === AST_NODE_TYPES.CallExpression) {
			const callee = node.callee
			if (callee.type === AST_NODE_TYPES.Identifier) {
				const name = callee.name
				// Check for i18n function calls
				if (I18N_SYMBOLS.includes(name as (typeof I18N_SYMBOLS)[number])) {
					hasI18n = true
				}
				if (I18N_CALL_PATTERNS.includes(name as (typeof I18N_CALL_PATTERNS)[number])) {
					hasI18n = true
					i18nUsages++
				}
			}
		}

		// Check for JSX elements: <IntlFormatted>
		if (node.type === AST_NODE_TYPES.JSXOpeningElement) {
			const name = node.name
			if (name.type === AST_NODE_TYPES.JSXIdentifier && name.name === 'IntlFormatted') {
				hasI18n = true
				i18nUsages++
			}
		}
	})

	return { hasI18n, i18nUsages }
}

/**
 * Extract plain text strings from template AST
 */
function extractTemplateStrings(templateContent: string): {
	plainStrings: string[]
	hasI18nPatterns: boolean
	i18nUsages: number
} {
	const plainStrings: string[] = []
	let hasI18nPatterns = false
	let i18nUsages = 0

	let ast: RootNode
	try {
		ast = parseTemplate(templateContent)
	} catch {
		// If parsing fails, return empty results
		return { plainStrings: [], hasI18nPatterns: false, i18nUsages: 0 }
	}

	walkTemplateAst(ast, (node) => {
		// Check for text nodes with plain text content
		if (node.type === NodeTypes.TEXT) {
			const textNode = node as TextNode
			if (isPlainTextString(textNode.content)) {
				plainStrings.push(textNode.content.trim())
			}
		}

		// Check element nodes
		if (node.type === NodeTypes.ELEMENT) {
			const elementNode = node as ElementNode
			const tagName = elementNode.tag

			// Check for IntlFormatted component
			if (tagName === 'IntlFormatted') {
				hasI18nPatterns = true
				i18nUsages++
			}

			// Check attributes for translatable content
			for (const prop of elementNode.props) {
				// Static attributes
				if (prop.type === NodeTypes.ATTRIBUTE) {
					const attrNode = prop as AttributeNode
					if (TRANSLATABLE_ATTRS.has(attrNode.name) && attrNode.value) {
						if (isPlainTextString(attrNode.value.content)) {
							plainStrings.push(`[${attrNode.name}]: ${attrNode.value.content}`)
						}
					}
				}

				// Directive attributes (v-bind, :attr, etc.)
				if (prop.type === NodeTypes.DIRECTIVE) {
					// Check for formatMessage or $t calls in directive expressions using AST
					if (prop.exp && prop.exp.type === NodeTypes.SIMPLE_EXPRESSION) {
						const callCount = countI18nCallsInExpression(prop.exp.content)
						if (callCount > 0) {
							hasI18nPatterns = true
							i18nUsages += callCount
						}
					}
				}
			}
		}

		// Check interpolation expressions for i18n calls using AST
		if (node.type === NodeTypes.INTERPOLATION) {
			if (node.content && node.content.type === NodeTypes.SIMPLE_EXPRESSION) {
				const callCount = countI18nCallsInExpression(node.content.content)
				if (callCount > 0) {
					hasI18nPatterns = true
					i18nUsages += callCount
				}
			}
		}
	})

	return { plainStrings, hasI18nPatterns, i18nUsages }
}

function analyzeVueFile(filePath: string): FileResult {
	const content = fs.readFileSync(filePath, 'utf-8')
	const { descriptor } = parseVue(content)

	const result: FileResult = {
		path: filePath,
		hasI18n: false,
		plainStrings: [],
		i18nUsages: 0,
	}

	// Analyze script content using AST
	const scriptContent = descriptor.script?.content || descriptor.scriptSetup?.content || ''
	if (scriptContent) {
		const scriptAnalysis = checkScriptForI18n(scriptContent)
		result.hasI18n = scriptAnalysis.hasI18n
		result.i18nUsages = scriptAnalysis.i18nUsages
	}

	// Analyze template content using AST
	if (descriptor.template?.content) {
		const templateAnalysis = extractTemplateStrings(descriptor.template.content)
		result.plainStrings = templateAnalysis.plainStrings
		if (templateAnalysis.hasI18nPatterns) {
			result.hasI18n = true
		}
		result.i18nUsages += templateAnalysis.i18nUsages
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

	// Directories to scan for Vue files
	const scanDirs = [
		'apps/frontend/src',
		'apps/app-frontend/src',
		'packages/ui/src',
	]

	if (!jsonOutput) {
		console.log()
		process.stdout.write(theme.muted('  Scanning Vue files... '))
	}

	const allFiles: string[] = []

	for (const dir of scanDirs) {
		const fullPath = path.join(rootDir, dir)
		if (fs.existsSync(fullPath)) {
			allFiles.push(...findVueFiles(fullPath))
		}
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
