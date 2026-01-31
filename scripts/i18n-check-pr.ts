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
import * as fs from 'fs'
import { execSync } from 'child_process'

interface LineRange {
	start: number
	end: number
}

// Attributes that typically contain translatable text
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

/**
 * Check if text is plain user-facing text that should be translated
 */
function isPlainTextString(text: string): boolean {
	const trimmed = text.trim()
	if (!trimmed) return false
	if (trimmed.length < 2) return false
	// Only punctuation/symbols/numbers
	if (/^[\s\d\-_./\\:;,!?@#$%^&*()[\]{}|<>+=~`'"]+$/.test(trimmed)) return false
	// Single identifier-like word (no spaces, likely a variable or class name)
	if (/^[a-z0-9_-]+$/i.test(trimmed) && !trimmed.includes(' ')) return false
	// Just a Vue interpolation
	if (/^\{\{.*\}\}$/.test(trimmed)) return false
	// No letters at all
	if (!/[a-zA-Z]/.test(trimmed)) return false
	// URLs
	if (/^https?:\/\//.test(trimmed)) return false
	// File/route paths
	if (/^\/[a-zA-Z_][\w\-/[\]]*$/.test(trimmed)) return false
	// Email addresses
	if (/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(trimmed)) return false
	return true
}

/**
 * Walk Vue template AST
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
 * Get changed line ranges for a file from git diff
 */
function getChangedLineRanges(filePath: string, baseRef: string): LineRange[] {
	const ranges: LineRange[] = []

	try {
		const diff = execSync(`git diff -U0 ${baseRef}...HEAD -- "${filePath}"`, {
			encoding: 'utf-8',
		})

		// Parse hunk headers: @@ -old,count +new,count @@
		// We want the +new,count part which tells us the new line ranges
		const hunkPattern = /^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@/gm
		let match

		while ((match = hunkPattern.exec(diff)) !== null) {
			const start = parseInt(match[1], 10)
			const count = match[2] ? parseInt(match[2], 10) : 1

			if (count > 0) {
				ranges.push({
					start,
					end: start + count - 1,
				})
			}
		}
	} catch {
		// If git diff fails, return empty ranges (skip the file)
	}

	return ranges
}

/**
 * Check if a line is within any of the changed ranges
 */
function isLineInChangedRanges(line: number, ranges: LineRange[]): boolean {
	return ranges.some((range) => line >= range.start && line <= range.end)
}

/**
 * Check a Vue file and output GitHub Actions warnings for untranslated text
 */
function checkFile(filePath: string, changedRanges: LineRange[]): number {
	let issueCount = 0

	let content: string
	try {
		content = fs.readFileSync(filePath, 'utf-8')
	} catch {
		return 0
	}

	let descriptor
	try {
		const parsed = parseVue(content)
		descriptor = parsed.descriptor
	} catch {
		return 0
	}

	if (!descriptor.template?.content) {
		return 0
	}

	const templateStartLine = descriptor.template.loc.start.line

	let ast: RootNode
	try {
		ast = parseTemplate(descriptor.template.content)
	} catch {
		return 0
	}

	walkTemplateAst(ast, (node) => {
		// Check text nodes
		if (node.type === NodeTypes.TEXT) {
			const textNode = node as TextNode
			if (isPlainTextString(textNode.content)) {
				const line = templateStartLine + textNode.loc.start.line - 1
				if (isLineInChangedRanges(line, changedRanges)) {
					const text = textNode.content.trim().replace(/\n/g, ' ').slice(0, 60)
					console.log(`::warning file=${filePath},line=${line}::Untranslated text: "${text}"`)
					issueCount++
				}
			}
		}

		// Check element attributes
		if (node.type === NodeTypes.ELEMENT) {
			const elementNode = node as ElementNode

			for (const prop of elementNode.props) {
				if (prop.type === NodeTypes.ATTRIBUTE) {
					const attrNode = prop as AttributeNode
					if (TRANSLATABLE_ATTRS.has(attrNode.name) && attrNode.value) {
						if (isPlainTextString(attrNode.value.content)) {
							const line = templateStartLine + attrNode.loc.start.line - 1
							if (isLineInChangedRanges(line, changedRanges)) {
								const text = attrNode.value.content.slice(0, 60)
								console.log(
									`::warning file=${filePath},line=${line}::Untranslated [${attrNode.name}]: "${text}"`,
								)
								issueCount++
							}
						}
					}
				}
			}
		}
	})

	return issueCount
}

/**
 * Main
 */
function main() {
	const args = process.argv.slice(2)

	// Parse --base argument
	let baseRef = 'origin/main'
	const baseIndex = args.indexOf('--base')
	if (baseIndex !== -1 && args[baseIndex + 1]) {
		baseRef = args[baseIndex + 1]
	}

	const files = args.filter((arg) => arg.endsWith('.vue'))

	if (files.length === 0) {
		console.log('No Vue files to check')
		process.exit(0)
	}

	console.log(`Checking ${files.length} Vue file(s) for untranslated text in diff...`)

	let totalIssues = 0
	for (const file of files) {
		if (fs.existsSync(file)) {
			const changedRanges = getChangedLineRanges(file, baseRef)
			if (changedRanges.length > 0) {
				totalIssues += checkFile(file, changedRanges)
			}
		}
	}

	if (totalIssues > 0) {
		console.log(`Found ${totalIssues} untranslated string(s) in changed lines`)
	} else {
		console.log('No untranslated strings found in changed lines')
	}

	// Always exit 0 - this is non-blocking
	process.exit(0)
}

main()
