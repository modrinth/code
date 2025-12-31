import { parse as parseVue } from '@vue/compiler-sfc'
import { parse as parseTs, AST_NODE_TYPES } from '@typescript-eslint/typescript-estree'
import type { TSESTree } from '@typescript-eslint/typescript-estree'
import chalk from 'chalk'
import * as fs from 'fs'
import * as path from 'path'

// i18n symbols that should be imported from @modrinth/ui
const I18N_SYMBOLS = ['useVIntl', 'defineMessage', 'defineMessages', 'IntlFormatted'] as const
type I18nSymbol = (typeof I18N_SYMBOLS)[number]

// formatMessage is special - it's destructured from useVIntl(), not directly imported
const FORMAT_MESSAGE = 'formatMessage'

// Valid import sources for i18n symbols
const VALID_IMPORT_SOURCES = ['@modrinth/ui']

// Directories to exclude from scanning
const EXCLUDED_DIRS = new Set(['node_modules', '.output', '.nuxt', 'dist', '.git', '.turbo'])

interface FileIssue {
	file: string
	symbol: string
	line: number
}

interface ImportInfo {
	symbol: string
	source: string
}

interface Usage {
	symbol: string
	line: number
}

const theme = {
	warning: chalk.yellow,
	error: chalk.red,
	success: chalk.green,
	muted: chalk.gray,
	highlight: chalk.white.bold,
	file: chalk.cyan,
}

/**
 * Recursively find all .vue and .ts files in directories
 */
function findFiles(dirs: string[]): string[] {
	const files: string[] = []

	function walk(dir: string) {
		if (!fs.existsSync(dir)) return

		const entries = fs.readdirSync(dir, { withFileTypes: true })
		for (const entry of entries) {
			const fullPath = path.join(dir, entry.name)

			if (entry.isDirectory()) {
				// Skip excluded directories and hidden directories
				if (!EXCLUDED_DIRS.has(entry.name) && !entry.name.startsWith('.')) {
					walk(fullPath)
				}
			} else if (entry.isFile()) {
				if (entry.name.endsWith('.vue') || entry.name.endsWith('.ts')) {
					// Skip .d.ts files
					if (!entry.name.endsWith('.d.ts')) {
						files.push(fullPath)
					}
				}
			}
		}
	}

	for (const dir of dirs) {
		walk(dir)
	}

	return files
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
 * Extract script content from Vue SFC
 */
function extractVueScript(content: string): { script: string; isTs: boolean } | null {
	try {
		const { descriptor } = parseVue(content)
		const scriptContent = descriptor.scriptSetup?.content || descriptor.script?.content
		if (!scriptContent) return null

		const lang = descriptor.scriptSetup?.lang || descriptor.script?.lang
		const isTs = lang === 'ts' || lang === 'tsx'

		return { script: scriptContent, isTs }
	} catch {
		return null
	}
}

/**
 * Walk AST and call visitor for each node
 */
function walkAst(node: TSESTree.Node, visitor: (node: TSESTree.Node) => void) {
	visitor(node)

	for (const key of Object.keys(node)) {
		const child = (node as Record<string, unknown>)[key]
		if (child && typeof child === 'object') {
			if (Array.isArray(child)) {
				for (const item of child) {
					if (item && typeof item === 'object' && 'type' in item) {
						walkAst(item as TSESTree.Node, visitor)
					}
				}
			} else if ('type' in child) {
				walkAst(child as TSESTree.Node, visitor)
			}
		}
	}
}

/**
 * Extract import information from AST
 */
function extractImports(ast: TSESTree.Program): ImportInfo[] {
	const imports: ImportInfo[] = []

	for (const node of ast.body) {
		if (node.type === AST_NODE_TYPES.ImportDeclaration) {
			const source = node.source.value as string
			for (const specifier of node.specifiers) {
				if (specifier.type === AST_NODE_TYPES.ImportSpecifier) {
					imports.push({
						symbol: specifier.imported.type === AST_NODE_TYPES.Identifier
							? specifier.imported.name
							: String(specifier.imported.value),
						source,
					})
				} else if (specifier.type === AST_NODE_TYPES.ImportDefaultSpecifier) {
					imports.push({
						symbol: specifier.local.name,
						source,
					})
				}
			}
		}
	}

	return imports
}

/**
 * Find usages of i18n symbols in AST
 */
function findUsages(ast: TSESTree.Program): Usage[] {
	const usages: Usage[] = []
	const localVariables = new Set<string>()

	// First pass: collect locally declared variables to avoid false positives
	walkAst(ast, (node) => {
		if (node.type === AST_NODE_TYPES.VariableDeclarator && node.id.type === AST_NODE_TYPES.Identifier) {
			localVariables.add(node.id.name)
		}
		if (node.type === AST_NODE_TYPES.FunctionDeclaration && node.id) {
			localVariables.add(node.id.name)
		}
	})

	// Second pass: find usages
	walkAst(ast, (node) => {
		// Check for call expressions: useVIntl(), defineMessage(), defineMessages(), formatMessage()
		if (node.type === AST_NODE_TYPES.CallExpression) {
			const callee = node.callee
			if (callee.type === AST_NODE_TYPES.Identifier) {
				const name = callee.name
				if (I18N_SYMBOLS.includes(name as I18nSymbol) || name === FORMAT_MESSAGE) {
					usages.push({
						symbol: name,
						line: callee.loc.start.line,
					})
				}
			}
		}

		// Check for JSX elements: <IntlFormatted>
		if (node.type === AST_NODE_TYPES.JSXOpeningElement) {
			const name = node.name
			if (name.type === AST_NODE_TYPES.JSXIdentifier && name.name === 'IntlFormatted') {
				usages.push({
					symbol: 'IntlFormatted',
					line: name.loc.start.line,
				})
			}
		}

		// Check for component references in Vue (e.g., components: { IntlFormatted })
		if (node.type === AST_NODE_TYPES.Property) {
			if (node.key.type === AST_NODE_TYPES.Identifier && node.key.name === 'IntlFormatted') {
				// This is defining IntlFormatted as a component, check if it's imported
				usages.push({
					symbol: 'IntlFormatted',
					line: node.key.loc.start.line,
				})
			}
		}
	})

	return usages
}

/**
 * Check if import source is valid for i18n symbols
 */
function isValidImportSource(source: string, filePath: string): boolean {
	// Direct import from @modrinth/ui
	if (VALID_IMPORT_SOURCES.includes(source)) {
		return true
	}

	// Relative imports within packages/ui are valid
	if (filePath.includes('packages/ui/') || filePath.includes('packages\\ui\\')) {
		if (source.startsWith('./') || source.startsWith('../')) {
			return true
		}
	}

	return false
}

/**
 * Analyze a single file for missing i18n imports
 */
function analyzeFile(filePath: string): FileIssue[] {
	const issues: FileIssue[] = []

	try {
		const content = fs.readFileSync(filePath, 'utf-8')
		let ast: TSESTree.Program | null = null

		if (filePath.endsWith('.vue')) {
			const scriptInfo = extractVueScript(content)
			if (!scriptInfo) return []
			ast = parseTsContent(scriptInfo.script, true)
		} else {
			ast = parseTsContent(content, filePath.endsWith('.tsx'))
		}

		if (!ast) return []

		const imports = extractImports(ast)
		const usages = findUsages(ast)

		// Build a map of imported symbols from valid sources
		const validImports = new Map<string, string>()
		for (const imp of imports) {
			if (isValidImportSource(imp.source, filePath)) {
				validImports.set(imp.symbol, imp.source)
			}
		}

		// Check if useVIntl is imported (for formatMessage validation)
		const hasUseVIntl = validImports.has('useVIntl')

		// Check each usage
		for (const usage of usages) {
			const symbol = usage.symbol

			if (symbol === FORMAT_MESSAGE) {
				// formatMessage is valid if useVIntl is imported
				if (!hasUseVIntl) {
					issues.push({
						file: filePath,
						symbol: `${symbol} (useVIntl not imported)`,
						line: usage.line,
					})
				}
			} else if (!validImports.has(symbol)) {
				issues.push({
					file: filePath,
					symbol,
					line: usage.line,
				})
			}
		}
	} catch {
		// Silent fail for unparsable files
	}

	return issues
}

/**
 * Main function
 */
function main() {
	const args = process.argv.slice(2)
	const verbose = args.includes('--verbose') || args.includes('-v')

	const rootDir = path.resolve(__dirname, '..')

	const dirsToScan = [
		path.join(rootDir, 'apps/frontend/src'),
		path.join(rootDir, 'apps/app-frontend/src'),
		path.join(rootDir, 'packages'),
	]

	console.log()
	process.stdout.write(theme.muted('  Scanning for i18n import issues... '))

	const files = findFiles(dirsToScan)
	console.log(theme.success(`found ${files.length} files`))

	const allIssues: FileIssue[] = []

	for (const file of files) {
		const issues = analyzeFile(file)
		allIssues.push(...issues)
	}

	console.log()

	if (allIssues.length === 0) {
		console.log(theme.success('  No missing i18n imports found!'))
		console.log()
		process.exit(0)
	}

	// Group issues by file
	const issuesByFile = new Map<string, FileIssue[]>()
	for (const issue of allIssues) {
		const existing = issuesByFile.get(issue.file) || []
		existing.push(issue)
		issuesByFile.set(issue.file, existing)
	}

	// Print issues
	for (const [file, issues] of issuesByFile) {
		const relativePath = path.relative(rootDir, file)
		console.log(theme.warning(`  ${relativePath}`))

		for (const issue of issues) {
			console.log(theme.muted(`    Line ${issue.line}: `) + theme.highlight(issue.symbol) + theme.muted(' is used but not imported'))
		}
		console.log()
	}

	// Summary
	console.log(theme.muted('  ─'.repeat(30)))
	console.log(
		theme.warning(`  Summary: ${issuesByFile.size} file(s) with ${allIssues.length} missing i18n import(s)`)
	)
	console.log()

	if (verbose) {
		console.log(theme.muted('  Tip: Import these symbols from @modrinth/ui'))
		console.log(theme.muted('  Example: import { useVIntl, defineMessages } from \'@modrinth/ui\''))
		console.log()
	}

	// Exit with 0 (warn only, don't block CI)
	process.exit(0)
}

main()
