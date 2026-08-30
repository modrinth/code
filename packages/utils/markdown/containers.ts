import { defineComarkPlugin } from 'comark/parse'

import { fencedBlockRule } from './fenced-block'

// reimpl/merge of https://mdit-plugins.github.io/layout.html and https://mdit-plugins.github.io/align.html

const alignNames = new Set(['left', 'center', 'right', 'justify'])

const alignRule = fencedBlockRule({
	marker: ':',
	ruleName: 'align',
	ruleBefore: 'fence',
	parentType: 'align',
	validate: (openText) => alignNames.has(openText.split(' ', 1)[0]),
	pushOpen: (state, openText, _hasSuffix, startLine, closeLine) => {
		const name = openText.split(' ', 1)[0]
		const tokenOpen = state.push('mdc_block_open', 'div', 1)
		tokenOpen.block = true
		tokenOpen.map = [startLine, closeLine]
		tokenOpen.attrSet('style', `text-align: ${name}`)
	},
	pushClose: (state) => {
		const tokenClose = state.push('mdc_block_close', 'div', -1)
		tokenClose.block = true
	},
})

type LayoutType = 'flex' | 'grid' | 'column'

const layoutBaseStyle: Record<LayoutType, string | undefined> = {
	flex: 'display:flex',
	grid: 'display:grid',
	column: undefined,
}

function isBoundary(ch: string | undefined): boolean {
	return ch === undefined || ch === '.' || ch === '#' || ch === ' '
}

type LayoutMarker =
	| { kind: 'container' | 'item'; type: LayoutType; depth: number; nameEnd: number }
	| { kind: 'end'; type: null; depth: number; nameEnd: number }

function matchesAt(src: string, pos: number, end: number, word: string): boolean {
	return pos + word.length <= end && src.startsWith(word, pos)
}

function boundaryAt(src: string, pos: number, end: number): boolean {
	return pos >= end || isBoundary(src[pos])
}

function parseLayoutMarker(src: string, start: number, end: number): LayoutMarker | null {
	if (src[start] !== '@') return null
	let pos = start + 1
	while (pos < end && src[pos] === '@') pos++
	const depth = pos - start

	if (matchesAt(src, pos, end, 'end') && boundaryAt(src, pos + 3, end)) {
		return { kind: 'end', type: null, depth, nameEnd: pos + 3 }
	}

	for (const type of ['flex', 'grid', 'column'] as const) {
		if (!matchesAt(src, pos, end, type)) continue
		const afterWord = pos + type.length
		if (afterWord < end && src[afterWord] === 's' && boundaryAt(src, afterWord + 1, end)) {
			return { kind: 'container', type, depth, nameEnd: afterWord + 1 }
		}
		if (boundaryAt(src, afterWord, end)) {
			return { kind: 'item', type, depth, nameEnd: afterWord }
		}
	}
	return null
}

interface LayoutModifiers {
	classes: string[]
	id?: string
	utilities: string[]
}

function parseLayoutModifiers(src: string, start: number, end: number): LayoutModifiers {
	const classes: string[] = []
	let id: string | undefined
	let pos = start
	while (pos < end) {
		const ch = src[pos]
		if (ch === '.' || ch === '#') {
			const tokenStart = pos + 1
			let tokenEnd = tokenStart
			while (tokenEnd < end && !isBoundary(src[tokenEnd])) tokenEnd++
			const token = src.slice(tokenStart, tokenEnd)
			if (token) {
				if (ch === '.') classes.push(token)
				else id = token
			}
			pos = tokenEnd
		} else if (ch === ' ') {
			const rest = src.slice(pos, end).trim()
			return { classes, id, utilities: rest ? rest.split(/\s+/) : [] }
		} else {
			break
		}
	}
	return { classes, id, utilities: [] }
}

/** Builds `{prefix}-{suffix}: property:value}` entries from a smaller suffix->value table. */
function expand(prefix: string, property: string, values: Record<string, string>): Record<string, string> {
	return Object.fromEntries(
		Object.entries(values).map(([suffix, value]) => [`${prefix}-${suffix}`, `${property}:${value}`]),
	)
}

const flexPosition = {
	start: 'flex-start',
	end: 'flex-end',
	center: 'center',
	between: 'space-between',
	around: 'space-around',
	evenly: 'space-evenly',
	stretch: 'stretch',
}
const plainPosition = { start: 'start', end: 'end', center: 'center', stretch: 'stretch' }
const plainSpacedPosition = { ...plainPosition, between: 'space-between', around: 'space-around', evenly: 'space-evenly' }
const breakPoints = {
	auto: 'auto',
	avoid: 'avoid',
	all: 'all',
	'avoid-page': 'avoid-page',
	page: 'page',
	left: 'left',
	right: 'right',
	column: 'column',
}

const utilityDeclarations: Record<string, string> = {
	...expand('flex', 'flex-direction', { row: 'row', col: 'column', 'row-reverse': 'row-reverse', 'col-reverse': 'column-reverse' }),
	...expand('flex', 'flex-wrap', { wrap: 'wrap', nowrap: 'nowrap', 'wrap-reverse': 'wrap-reverse' }),
	...expand('flex', 'flex', { 1: '1 1 0%', auto: '1 1 auto', initial: '0 1 auto', none: 'none' }),
	grow: 'flex-grow:1',
	'grow-0': 'flex-grow:0',
	shrink: 'flex-shrink:1',
	'shrink-0': 'flex-shrink:0',
	'order-first': 'order:-9999',
	'order-last': 'order:9999',
	'order-none': 'order:0',
	...expand('grid-flow', 'grid-auto-flow', {
		row: 'row',
		col: 'column',
		dense: 'dense',
		'row-dense': 'row dense',
		'col-dense': 'column dense',
	}),
	...expand('auto-cols', 'grid-auto-columns', {
		auto: 'auto',
		min: 'min-content',
		max: 'max-content',
		fr: 'minmax(0,1fr)',
	}),
	...expand('auto-rows', 'grid-auto-rows', {
		auto: 'auto',
		min: 'min-content',
		max: 'max-content',
		fr: 'minmax(0,1fr)',
	}),
	'grid-cols-none': 'grid-template-columns:none',
	'grid-rows-none': 'grid-template-rows:none',
	'col-span-full': 'grid-column:1 / -1',
	'row-span-full': 'grid-row:1 / -1',
	...expand('justify', 'justify-content', flexPosition),
	...expand('justify-items', 'justify-items', plainPosition),
	...expand('justify-self', 'justify-self', { auto: 'auto', ...plainPosition }),
	...expand('content', 'align-content', flexPosition),
	...expand('items', 'align-items', { ...flexPosition, baseline: 'baseline' }),
	...expand('self', 'align-self', { auto: 'auto', ...flexPosition, baseline: 'baseline' }),
	...expand('place-content', 'place-content', plainSpacedPosition),
	...expand('place-items', 'place-items', plainPosition),
	...expand('place-self', 'place-self', { auto: 'auto', ...plainPosition }),
	'aspect-auto': 'aspect-ratio:auto',
	'aspect-square': 'aspect-ratio:1 / 1',
	'aspect-video': 'aspect-ratio:16 / 9',
	...expand('break-after', 'break-after', breakPoints),
	...expand('break-before', 'break-before', breakPoints),
	'break-inside-auto': 'break-inside:auto',
	'break-inside-avoid': 'break-inside:avoid',
	'break-inside-avoid-page': 'break-inside:avoid-page',
	'break-inside-avoid-column': 'break-inside:avoid-column',
	'gap-px': 'gap:1px',
	'gap-x-px': 'column-gap:1px',
	'gap-y-px': 'row-gap:1px',
}

function resolvePatternUtility(name: string): string {
	const numbered = (prefix: string): number | undefined => {
		if (!name.startsWith(prefix)) return undefined
		const n = Number(name.slice(prefix.length))
		return Number.isInteger(n) && n >= 0 ? n : undefined
	}

	let n = numbered('gap-x-')
	if (n !== undefined) return `column-gap:${n * 0.25}rem`
	n = numbered('gap-y-')
	if (n !== undefined) return `row-gap:${n * 0.25}rem`
	n = numbered('gap-')
	if (n !== undefined) return `gap:${n * 0.25}rem`
	n = numbered('order-')
	if (n !== undefined) return `order:${n}`
	n = numbered('grid-cols-')
	if (n !== undefined) return `grid-template-columns:repeat(${n},minmax(0,1fr))`
	n = numbered('grid-rows-')
	if (n !== undefined) return `grid-template-rows:repeat(${n},minmax(0,1fr))`
	n = numbered('col-span-')
	if (n !== undefined) return `grid-column:span ${n} / span ${n}`
	n = numbered('col-start-')
	if (n !== undefined) return `grid-column-start:${n}`
	n = numbered('col-end-')
	if (n !== undefined) return `grid-column-end:${n}`
	n = numbered('row-span-')
	if (n !== undefined) return `grid-row:span ${n} / span ${n}`
	n = numbered('row-start-')
	if (n !== undefined) return `grid-row-start:${n}`
	n = numbered('row-end-')
	if (n !== undefined) return `grid-row-end:${n}`
	n = numbered('columns-')
	if (n !== undefined) return `columns:${n}`
	return ''
}

function resolveUtility(name: string): string {
	return utilityDeclarations[name] ?? resolvePatternUtility(name)
}

function formatUtilities(utilities: string[], baseStyle?: string): string | undefined {
	const declarations = baseStyle ? [baseStyle] : []
	for (const utility of utilities) {
		const declaration = resolveUtility(utility)
		if (declaration) declarations.push(declaration)
	}
	return declarations.length ? declarations.join(';') : undefined
}

function findLayoutContainerEnd(state: any, from: number, end: number, depth: number): number {
	let openCount = 1
	for (let line = from; line < end; line++) {
		const lineStart = state.bMarks[line] + state.tShift[line]
		const lineEnd = state.eMarks[line]
		if (lineStart >= lineEnd || state.src[lineStart] !== '@') continue
		const marker = parseLayoutMarker(state.src, lineStart, lineEnd)
		if (!marker || marker.depth !== depth) continue
		if (marker.kind === 'container') openCount++
		else if (marker.kind === 'end') {
			openCount--
			if (openCount === 0) return line
		}
	}
	return end
}

function findLayoutItemEnd(state: any, from: number, end: number, type: LayoutType, depth: number): number {
	let nestedOpen = 0
	for (let line = from; line < end; line++) {
		const lineStart = state.bMarks[line] + state.tShift[line]
		const lineEnd = state.eMarks[line]
		if (lineStart >= lineEnd || state.src[lineStart] !== '@') continue
		const marker = parseLayoutMarker(state.src, lineStart, lineEnd)
		if (!marker || marker.depth !== depth) continue
		if (marker.kind === 'container') nestedOpen++
		else if (marker.kind === 'end') {
			if (nestedOpen > 0) nestedOpen--
			else return line
		} else if (marker.kind === 'item' && marker.type === type && nestedOpen === 0) {
			return line
		}
	}
	return end
}

const layoutContext = Symbol('layout')

function pushLayoutDiv(
	state: any,
	startLine: number,
	endLine: number,
	attrs: { style?: string; class?: string; id?: string },
) {
	const tokenOpen = state.push('mdc_block_open', 'div', 1)
	tokenOpen.block = true
	tokenOpen.map = [startLine, endLine]
	if (attrs.style) tokenOpen.attrSet('style', attrs.style)
	if (attrs.class) tokenOpen.attrSet('class', attrs.class)
	if (attrs.id) tokenOpen.attrSet('id', attrs.id)

	state.md.block.tokenize(state, startLine + 1, endLine)

	const tokenClose = state.push('mdc_block_close', 'div', -1)
	tokenClose.block = true
}

function layoutContainerRule(md: any) {
	md.block.ruler.before(
		'fence',
		'layout_container',
		(state: any, startLine: number, endLine: number, silent: boolean) => {
			const lineStart = state.bMarks[startLine] + state.tShift[startLine]
			const lineEnd = state.eMarks[startLine]
			const marker = parseLayoutMarker(state.src, lineStart, lineEnd)
			if (!marker || marker.kind !== 'container') return false
			if (silent) return true

			const indent = state.sCount[startLine]
			const modifiers = parseLayoutModifiers(state.src, marker.nameEnd, lineEnd)
			const closeLine = findLayoutContainerEnd(state, startLine + 1, endLine, marker.depth)

			const oldParent = state.parentType
			const oldLineMax = state.lineMax
			const oldIndent = state.blkIndent
			const oldContext = state.env[layoutContext]
			state.parentType = 'layout_container'
			state.lineMax = closeLine
			state.blkIndent = indent
			state.env[layoutContext] = { type: marker.type, depth: marker.depth }

			pushLayoutDiv(state, startLine, closeLine, {
				style: formatUtilities(modifiers.utilities, layoutBaseStyle[marker.type]),
				class: modifiers.classes.join(' ') || undefined,
				id: modifiers.id,
			})

			state.env[layoutContext] = oldContext
			state.parentType = oldParent
			state.lineMax = oldLineMax
			state.blkIndent = oldIndent
			state.line = closeLine < endLine ? closeLine + 1 : closeLine
			return true
		},
		{ alt: ['paragraph', 'reference', 'blockquote', 'list', 'layout_container'] },
	)
}

function layoutItemRule(md: any) {
	md.block.ruler.before(
		'paragraph',
		'layout_item',
		(state: any, startLine: number, endLine: number, silent: boolean) => {
			const context = state.env[layoutContext]
			if (!context) return false

			const lineStart = state.bMarks[startLine] + state.tShift[startLine]
			const lineEnd = state.eMarks[startLine]
			const marker = parseLayoutMarker(state.src, lineStart, lineEnd)
			if (!marker || marker.kind !== 'item' || marker.type !== context.type || marker.depth !== context.depth)
				return false
			if (silent) return true

			const indent = state.sCount[startLine]
			const modifiers = parseLayoutModifiers(state.src, marker.nameEnd, lineEnd)
			const closeLine = findLayoutItemEnd(state, startLine + 1, endLine, marker.type, marker.depth)

			const styleDeclarations: string[] = []
			if (marker.type === 'column' && modifiers.classes.includes('span-all')) {
				styleDeclarations.push('column-span:all')
			}
			for (const utility of modifiers.utilities) {
				const declaration = resolveUtility(utility)
				if (declaration) styleDeclarations.push(declaration)
			}

			const oldParent = state.parentType
			const oldLineMax = state.lineMax
			const oldIndent = state.blkIndent
			state.parentType = 'layout_item'
			state.lineMax = closeLine
			state.blkIndent = indent
			state.env[layoutContext] = undefined

			pushLayoutDiv(state, startLine, closeLine, {
				style: styleDeclarations.length ? styleDeclarations.join(';') : undefined,
				class: modifiers.classes.filter((cls) => cls !== 'span-all').join(' ') || undefined,
				id: modifiers.id,
			})

			state.env[layoutContext] = context
			state.parentType = oldParent
			state.lineMax = oldLineMax
			state.blkIndent = oldIndent
			state.line = closeLine
			return true
		},
		{ alt: ['paragraph', 'reference', 'blockquote', 'list', 'layout_container'] },
	)
}

export const containers = defineComarkPlugin(() => ({
	name: 'containers',
	markdownItPlugins: [alignRule, layoutContainerRule, layoutItemRule],
}))
