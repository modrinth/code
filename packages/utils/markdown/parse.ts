import { createHtmlRenderer } from '@comark/html'
import type { ComponentRenderFn } from '@comark/html/render'
import type { ComarkPlugin, ElementNode, MarkdownDocument, Node } from 'comark'
import { parseMarkdown } from 'comark'
import { defineComarkPlugin } from 'comark/parse'
import githubAlert from 'comark/plugins/alert'
import attributes from 'comark/plugins/attributes'
import binding from 'comark/plugins/binding'
import components from 'comark/plugins/components'
import emoji from 'comark/plugins/emoji'
import footnotes from 'comark/plugins/footnotes'
import html from 'comark/plugins/html'
import math from 'comark/plugins/math'
import mermaid from 'comark/plugins/mermaid'
import punctuation from 'comark/plugins/punctuation'
import taskList from 'comark/plugins/task-list'
import { visitAsync } from 'comark/utils'

import { alert } from './alert'
import { inlineMarkers } from './custom-syntax'
import { detailsRegion } from './details-region'
import { embedSyntax } from './embeds'
import { htmlBlock } from './html-block'
import { resolveMedia } from './media'
import { security } from './security'

export { security, securityOptions } from './security'

export type { ElementNode, MarkdownDocument, Node } from 'comark'
export { visit } from 'comark/utils'

export function getFenceLanguage(preNode: ElementNode): string | undefined {
	return typeof preNode[1]?.language === 'string' ? (preNode[1].language as string) : undefined
}

export function getFenceCodeText(preNode: ElementNode): string {
	const codeNode = preNode[2] as ElementNode | undefined
	return typeof codeNode?.[2] === 'string' ? codeNode[2] : ''
}

export const defaultPlugins: ComarkPlugin[] = [
	// breaks(), //TODO: make this an option?
	emoji(),
	punctuation(),
	footnotes(),

	binding(),
	htmlBlock(),
	detailsRegion(),
	inlineMarkers([
		{ markers: ['^'], tag: 'sup' },
		{ markers: ['~'], tag: 'sub' },
		{ markers: ['!!', '||'], tag: 'span', attrs: { class: 'spoiler', tabindex: '0' } },
		{ markers: ['=='], tag: 'mark' },
		{ markers: ['+'], tag: 'ins' },
	]),
	alert(),
	math(),
	mermaid(),
	embedSyntax(),
	resolveMedia(),
	security(),
	// shiki(), // needs dep + we have highlight.js
	// toc() // would be cool to have but would need heavy changes to project pages
]

export function createRenderer(extraComponents: Record<string, ComponentRenderFn> = {}) {
	const renderer = createHtmlRenderer({
		linkify: true,
		registerDefaultPlugins: false,
		autoClose: false,
		plugins: defaultPlugins,
		components: extraComponents,
	})
	return (markdown: string): Promise<string> => renderer(markdown)
}

export const renderString = createRenderer()

export function parseDocument(markdown: string): Promise<MarkdownDocument> {
	return parseMarkdown(markdown, {
		linkify: true,
		registerDefaultPlugins: false,
		autoClose: false,
		plugins: defaultPlugins,
	})
}

const blogRelativeUrlAttrs: Record<string, string> = { a: 'href', img: 'src' }

function createBlogUrlRewritePlugin(baseUrl: string, siteUrl: string) {
	return defineComarkPlugin(() => ({
		name: 'blog-relative-urls',
		async post(state) {
			await visitAsync(
				state.tree,
				(node) => typeof node !== 'string' && node[0] !== null,
				(node) => {
					const element = node as ElementNode
					const attrName = blogRelativeUrlAttrs[element[0].toLowerCase()]
					const value = attrName ? element[1][attrName] : undefined
					if (typeof value !== 'string' || !value) return

					element[1][attrName] = new URL(value, baseUrl).href.replace(siteUrl, '')
				},
			)
		},
	}))()
}

export function createBlogHtmlRenderer(baseUrl: string, siteUrl: string) {
	const renderer = createHtmlRenderer({
		linkify: true,
		registerDefaultPlugins: false,
		plugins: [
			githubAlert(),
			attributes(),
			components(),
			html(),
			taskList(),
			createBlogUrlRewritePlugin(baseUrl, siteUrl),
		],
	})
	return (markdown: string): Promise<string> => renderer(markdown)
}


export const basicPlugins = [
	resolveMedia(),
	security({
		allowedProtocols: ['https', 'mailto'],
		allowedTags: ['a', 'strong', 'em', 'code', 'br'],
		tagFallback: (element) => {
			const text = element
				.slice(2)
				.filter((child): child is string => typeof child === 'string')
				.join('')
			return text ? text : false
		},
	}),
]

export function forceLinkTarget(target: string) {
	return defineComarkPlugin(() => ({
		name: 'force-link-target',
		async post(state) {
			await visitAsync(
				state.tree,
				(node) => typeof node !== 'string' && node[0] === 'a',
				(node) => {
					;(node as ElementNode)[1].target = target
				},
			)
		},
	}))()
}
