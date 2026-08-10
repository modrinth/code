
import { existsSync, mkdirSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

import type { CallExpression, Node, ReturnStatement } from 'ts-morph'
import { Project, SyntaxKind } from 'ts-morph'

const __dirname = dirname(fileURLToPath(import.meta.url))
const PACKAGE_ROOT = join(__dirname, '..')
const STAGES_GLOB = join(PACKAGE_ROOT, 'src/data/stages/*.tsx')
const OUT_DIR = join(PACKAGE_ROOT, '.checklist-docs')

const FACTORY_INPUT_TYPES: Record<string, string> = {
	stage: 'Stage',
	toggle: 'Toggle',
	toggleSwitch: 'Switch',
	check: 'Checkbox',
	button: 'Button',
	group: 'Group',
	externalGroup: 'Group (no input, external state)',
	option: 'Option',
	dropdown: 'Dropdown',
	text: 'Text Input',
	markdown: 'Markdown Editor',
	appComponent: 'Custom Component',
}
const FACTORY_NAMES = new Set(Object.keys(FACTORY_INPUT_TYPES))

interface NodeInfo {
	type: string
	id?: string
	label?: string
	shown?: string
	suggestedStatus?: string
	messagePath?: string
	fix?: string
	priority?: string
	hint?: string
	guidance?: string
	navigate?: string
	children: NodeInfo[]
}

function unwrapParens(node: Node): Node {
	let current = node
	while (current.getKind() === SyntaxKind.ParenthesizedExpression) {
		current = (current as unknown as { getExpression(): Node }).getExpression()
	}
	return current
}

function normalizeCode(text: string): string {
	return text.replace(/\s+/g, ' ').trim()
}

function resolveRelativeMessagePath(name: string, statePath: string[]): string {
	if (name.startsWith('/')) return `checklist/messages${name}`
	const parts = [...statePath.slice(0, -1), ...name.split('/')]
	const normalized = parts.reduce<string[]>((acc, p) => {
		if (p === '..') acc.pop()
		else if (p) acc.push(p)
		return acc
	}, [])
	return `checklist/messages/${normalized.join('/')}`
}

function autoMessagePath(statePath: string[]): string {
	return `checklist/messages/${statePath.join('/')}`
}

function literalText(arg: Node | undefined): string | undefined {
	if (!arg) return undefined
	if (arg.getKind() === SyntaxKind.StringLiteral || arg.getKind() === SyntaxKind.NoSubstitutionTemplateLiteral) {
		return (arg as unknown as { getLiteralText(): string }).getLiteralText()
	}
	return undefined
}

function findDocsComment(node: Node): string | undefined {
	let current: Node | undefined = node
	while (current) {
		const ranges = current.getLeadingCommentRanges()
		for (const range of ranges) {
			const text = range.getText()
			const match = text.match(/@docs\s+([\s\S]*?)(?:\*\/)?\s*$/)
			if (match) {
				return match[1]
					.replace(/^\/\*+\s*|\s*\*+\/$/g, '')
					.replace(/^\/\/\s*/g, '')
					.trim()
			}
		}
		const parent = current.getParent()
		if (!parent || current.getKind() === SyntaxKind.ExpressionStatement) break
		current = parent
	}
	return undefined
}

function unwindChain(expr: CallExpression): { root: CallExpression | undefined; calls: CallExpression[] } {
	const calls: CallExpression[] = []
	let current: Node = expr
	while (current.getKind() === SyntaxKind.CallExpression) {
		const call = current as CallExpression
		const callee = call.getExpression()
		if (callee.getKind() === SyntaxKind.PropertyAccessExpression) {
			calls.unshift(call)
			current = (callee as unknown as { getExpression(): Node }).getExpression()
		} else if (callee.getKind() === SyntaxKind.Identifier) {
			return { root: call, calls }
		} else {
			break
		}
	}
	return { root: undefined, calls }
}

const FUNCTION_LIKE_KINDS = new Set([
	SyntaxKind.ArrowFunction,
	SyntaxKind.FunctionExpression,
	SyntaxKind.FunctionDeclaration,
	SyntaxKind.MethodDeclaration,
])

function getOwnReturnStatements(fn: Node): ReturnStatement[] {
	return fn.getDescendantsOfKind(SyntaxKind.ReturnStatement).filter((ret) => {
		let current: Node | undefined = ret.getParent()
		while (current && current !== fn) {
			if (FUNCTION_LIKE_KINDS.has(current.getKind())) return false
			current = current.getParent()
		}
		return current === fn
	})
}

function resolveLocalHelperCall(call: CallExpression, path: string[]): NodeInfo | undefined {
	const callee = call.getExpression()
	if (callee.getKind() !== SyntaxKind.Identifier) return undefined
	const name = callee.getText()
	const sourceFile = call.getSourceFile()
	const fn = sourceFile
		.getDescendantsOfKind(SyntaxKind.FunctionDeclaration)
		.find((f) => f.getName() === name)
	if (!fn) return { type: 'UNRESOLVED', children: [] }
	const returnStatements = getOwnReturnStatements(fn)
	for (const ret of returnStatements) {
		const returnExpr = ret.getExpression() ? unwrapParens(ret.getExpression()!) : undefined
		if (returnExpr && returnExpr.getKind() === SyntaxKind.CallExpression) {
			const resolved = resolveNode(returnExpr as CallExpression, path)
			if (resolved) return resolved
		}
	}
	return { type: 'UNRESOLVED', children: [] }
}

function resolveChildEntry(rawArg: Node, path: string[]): NodeInfo[] {
	const arg = unwrapParens(rawArg)
	if (arg.getKind() === SyntaxKind.CallExpression) {
		const call = arg as CallExpression
		const callee = call.getExpression()
		if (callee.getKind() === SyntaxKind.Identifier && !FACTORY_NAMES.has(callee.getText())) {
			const calleeName = callee.getText()
			if (calleeName === 'computed' || calleeName === 'ref') {
				const docs = findDocsComment(call)
				return [
					{
						type: docs ? `Dynamic children: ${docs}` : 'UNRESOLVED',
						children: [],
					},
				]
			}
			return [resolveLocalHelperCall(call, path)!]
		}
		const resolved = resolveNode(call, path)
		return [resolved ?? { type: 'UNRESOLVED', children: [] }]
	}
	if (arg.getKind() === SyntaxKind.SpreadElement) {
		const spreadArg = (arg as unknown as { getExpression(): Node }).getExpression()
		return resolveChildEntry(spreadArg, path)
	}
	if (arg.getKind() === SyntaxKind.ArrayLiteralExpression) {
		return arg
			.getChildrenOfKind(SyntaxKind.SyntaxList)
			.flatMap((list) => list.getChildren().flatMap((c) => resolveChildEntry(c, path)))
	}
	if (arg.getKind() === SyntaxKind.ArrowFunction || arg.getKind() === SyntaxKind.FunctionExpression) {
		return []
	}
	if (arg.getKind() === SyntaxKind.StringLiteral) {
		return []
	}
	return [{ type: 'UNRESOLVED', children: [] }]
}

function resolveNode(expr: CallExpression, path: string[]): NodeInfo | undefined {
	const { root, calls } = unwindChain(expr)
	if (!root) return undefined
	const factoryName = root.getExpression().getText()
	const type = FACTORY_INPUT_TYPES[factoryName]
	if (!type) return undefined

	const rootArgs = root.getArguments()
	const info: NodeInfo = {
		type,
		id: literalText(rootArgs[0]),
		label: literalText(rootArgs[1]),
		children: [],
	}
	const nodePath = info.id ? [...path, info.id] : path

	for (const call of calls) {
		const callee = call.getExpression()
		if (callee.getKind() !== SyntaxKind.PropertyAccessExpression) continue
		const methodName = (callee as unknown as { getName(): string }).getName()
		const args = call.getArguments()

		switch (methodName) {
			case 'shown':
				info.shown = findDocsComment(call) ?? (args[0] ? normalizeCode(args[0].getText()) : undefined)
				break
			case 'suggestedStatus':
				info.suggestedStatus = literalText(args[0])
				break
			case 'message': {
				const arg0 = args[0]
				const literal = literalText(arg0)
				if (literal !== undefined) {
					info.messagePath = resolveRelativeMessagePath(literal, nodePath)
				} else if (
					arg0 &&
					(arg0.getKind() === SyntaxKind.ArrowFunction ||
						arg0.getKind() === SyntaxKind.FunctionExpression)
				) {
					const paramCount = (
						arg0 as unknown as { getParameters(): unknown[] }
					).getParameters().length
					info.messagePath =
						paramCount >= 1 ? autoMessagePath(nodePath) : normalizeCode(arg0.getText())
				} else if (arg0) {
					info.messagePath = normalizeCode(arg0.getText())
				} else {
					info.messagePath = autoMessagePath(nodePath)
				}
				break
			}
			case 'fix':
				info.fix = findDocsComment(call) ?? (args[0] ? normalizeCode(args[0].getText()) : 'UNRESOLVED')
				break
			case 'priority':
				info.priority = args[0] ? normalizeCode(args[0].getText()) : undefined
				break
			case 'hint':
				info.hint = literalText(args[0])
				break
			case 'guidance':
				info.guidance = literalText(args[0])
				break
			case 'navigate':
				info.navigate = literalText(args[0])
				break
			case 'children':
				for (const arg of args) {
					info.children.push(...resolveChildEntry(arg, nodePath))
				}
				break
			default:
				break
		}
	}

	return info
}

function pruneForJson(node: NodeInfo): Record<string, unknown> {
	const out: Record<string, unknown> = { type: node.type }
	if (node.id) out.id = node.id
	if (node.label) out.label = node.label
	if (node.shown) out.shown = node.shown
	if (node.fix) out.fix = node.fix
	if (node.suggestedStatus) out.suggestedStatus = node.suggestedStatus
	if (node.messagePath) out.messagePath = node.messagePath
	if (node.priority) out.priority = node.priority
	if (node.hint) out.hint = node.hint
	if (node.guidance) out.guidance = node.guidance
	if (node.navigate) out.navigate = node.navigate
	if (node.children.length > 0) out.children = node.children.map(pruneForJson)
	return out
}

function main() {
	const project = new Project({
		tsConfigFilePath: join(PACKAGE_ROOT, 'tsconfig.json'),
	})
	project.addSourceFilesAtPaths(STAGES_GLOB)

	const jsonOutput: Record<string, NodeInfo | undefined> = {}

	for (const sourceFile of project.getSourceFiles(STAGES_GLOB)) {
		const stageFileName = sourceFile.getBaseNameWithoutExtension()
		const defaultExport =
			sourceFile.getFunction((f) => f.isDefaultExport()) ??
			sourceFile.getExportAssignments()[0]?.getExpression()

		let returnExpr: Node | undefined
		if (defaultExport && 'getDescendantsOfKind' in defaultExport) {
			const returnStatements = getOwnReturnStatements(defaultExport as unknown as Node)
			const lastReturnExpr = returnStatements[returnStatements.length - 1]?.getExpression()
			returnExpr = lastReturnExpr ? unwrapParens(lastReturnExpr) : undefined
		}

		if (!returnExpr || returnExpr.getKind() !== SyntaxKind.CallExpression) {
			console.warn(`[skip] ${stageFileName}: could not find a default-export stage() call`)
			continue
		}

		const stageNode = resolveNode(returnExpr as CallExpression, [])
		if (!stageNode) {
			console.warn(`[skip] ${stageFileName}: default export didn't resolve to a stage() chain`)
			continue
		}

		jsonOutput[stageFileName] = stageNode
	}

	if (!existsSync(OUT_DIR)) mkdirSync(OUT_DIR, { recursive: true })

	const prunedJsonOutput = Object.fromEntries(
		Object.entries(jsonOutput).map(([key, node]) => [key, node ? pruneForJson(node) : node]),
	)
	const jsonPath = join(OUT_DIR, 'checklist-structure.json')
	writeFileSync(jsonPath, JSON.stringify(prunedJsonOutput, null, 2))

	console.log(`Wrote ${Object.keys(jsonOutput).length} stages to:`)
	console.log(`  ${jsonPath}`)
}

main()
