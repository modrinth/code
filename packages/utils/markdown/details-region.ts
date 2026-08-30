import type { ElementNode } from 'comark'
import { defineComarkPlugin } from 'comark/parse'
import { visitAsync } from 'comark/utils'

import { fencedBlockRule } from './fenced-block'

const detailsRegionRule = fencedBlockRule({
	marker: '+',
	ruleName: 'details_region',
	ruleBefore: 'fence',
	parentType: 'details_region',
	suffixChar: '>',
	pushOpen: (state, openText, isOpen, startLine, closeLine) => {
		const detailsOpen = state.push('mdc_block_open', 'details', 1)
		detailsOpen.block = true
		detailsOpen.map = [startLine, closeLine]
		if (isOpen) detailsOpen.attrSet('open', '')

		const summaryOpen = state.push('mdc_block_open', 'summary', 1)
		summaryOpen.block = true
		if (openText) {
			const paragraphOpen = state.push('paragraph_open', 'p', 1)
			paragraphOpen.map = [startLine, startLine + 1]
			const summaryInline = state.push('inline', '', 0)
			summaryInline.content = openText
			summaryInline.map = [startLine, startLine + 1]
			summaryInline.children = []
			state.push('paragraph_close', 'p', -1)
		}
		const summaryClose = state.push('mdc_block_close', 'summary', -1)
		summaryClose.block = true
	},
	pushClose: (state) => {
		const detailsClose = state.push('mdc_block_close', 'details', -1)
		detailsClose.block = true
	},
})

export const detailsRegion = defineComarkPlugin(() => ({
	name: 'details-region',
	markdownItPlugins: [detailsRegionRule],
	async post(state) {
		await visitAsync(
			state.tree,
			(node) => typeof node !== 'string' && (node[0] === 'details' || node[0] === 'summary'),
			(node) => {
				const element = node as ElementNode
				const children = element.slice(2)
				if (children.length === 1 && typeof children[0] === 'string') {
					element.length = 2
					element.push(['p', {}, children[0]])
				}
			},
		)

		await visitAsync(
			state.tree,
			(node) => typeof node !== 'string' && node[0] === 'details',
			(node) => {
				const element = node as ElementNode
				const children = element.slice(2)
				const hasSummary = children.some((child) => Array.isArray(child) && child[0] === 'summary')
				if (!hasSummary) element.splice(2, 0, ['summary', {}, 'Details'])
			},
		)

		await visitAsync(
			state.tree,
			(node) => typeof node !== 'string' && node[0] === 'details',
			(node) => {
				const element = node as ElementNode
				const children = element.slice(2)
				const contentChildren = children.filter(
					(child) => !(Array.isArray(child) && child[0] === 'summary'),
				)
				const isEmpty = contentChildren.every(
					(child) => typeof child === 'string' && !child.trim(),
				)
				if (isEmpty) element[1].class = [element[1].class, 'empty'].filter(Boolean).join(' ')
			},
		)
	},
}))
