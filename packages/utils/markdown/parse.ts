import { createHtmlRenderer } from '@comark/html'
import type { ComponentRenderFn } from '@comark/html/render'
import type { ComarkPlugin, ElementNode, MarkdownDocument, Node } from 'comark'
import { parseMarkdown } from 'comark'
import { defineComarkPlugin } from 'comark/parse'
import alert from 'comark/plugins/alert'
import attributes from 'comark/plugins/attributes'
import binding from 'comark/plugins/binding'
import breaks from 'comark/plugins/breaks'
import components from 'comark/plugins/components'
import emoji from 'comark/plugins/emoji'
import footnotes from 'comark/plugins/footnotes'
import html from 'comark/plugins/html'
import punctuation from 'comark/plugins/punctuation'
import taskList from 'comark/plugins/task-list'
import { visitAsync } from 'comark/utils'

import { modrinthEmbedSyntax } from './embeds'
import { modrinthResolveMedia } from './media'
import { modrinthSecurity } from './security'

export { modrinthSecurity, securityOptions } from './security'

export type { ElementNode, MarkdownDocument, Node } from 'comark'
export { visit } from 'comark/utils'

export function getFenceLanguage(preNode: ElementNode): string | undefined {
	return typeof preNode[1]?.language === 'string' ? (preNode[1].language as string) : undefined
}

export function getFenceCodeText(preNode: ElementNode): string {
	const codeNode = preNode[2] as ElementNode | undefined
	return typeof codeNode?.[2] === 'string' ? codeNode[2] : ''
}

export const modrinthPlugins: ComarkPlugin[] = [
	alert(),
	attributes(), // safe cuz we filter attributes, ends up being a nice shorthand for any places people want to use the attributes we allow?
	binding(),
	breaks(), // makes newlines work like newlines
	components(),
	emoji(),
	footnotes(),
	// frontmatter()
	// heading()
	html(),
	// json-render()
	// math(), // needs dep
	// mermaid(), // needs dep
	punctuation(),
	// rangi() // needs dep + we have highlight.js + if we do want to switch I'd say shiki is better
	modrinthEmbedSyntax(),
	modrinthResolveMedia(),
	modrinthSecurity(),
	// shiki(), // needs dep + we have highlight.js
	// summary() // irrelevant
	taskList(),
	// toc() // would be cool to have but would need heavy changes to project pages
]

export function createModrinthHtmlRenderer(
	extraComponents: Record<string, ComponentRenderFn> = {},
) {
	const renderer = createHtmlRenderer({
		linkify: true,
		registerDefaultPlugins: false,
		plugins: modrinthPlugins,
		components: extraComponents,
	})
	return (markdown: string): Promise<string> => renderer(markdown)
}

export const renderString = createModrinthHtmlRenderer()

export function parseModrinthMarkdown(markdown: string): Promise<MarkdownDocument> {
	return parseMarkdown(markdown, {
		linkify: true,
		registerDefaultPlugins: false,
		plugins: modrinthPlugins,
	})
}

const blogRelativeUrlAttrs: Record<string, string> = { a: 'href', img: 'src' }

function createBlogUrlRewritePlugin(baseUrl: string, siteUrl: string) {
	return defineComarkPlugin(() => ({
		name: 'modrinth-blog-relative-urls',
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
			alert(),
			attributes(),
			components(),
			html(),
			taskList(),
			createBlogUrlRewritePlugin(baseUrl, siteUrl),
		],
	})
	return (markdown: string): Promise<string> => renderer(markdown)
}

const basicMarkdownAllowedTags = ['a', 'strong', 'em', 'code', 'br']

export const modrinthBasicPlugins = [
	modrinthResolveMedia(),
	modrinthSecurity({
		allowedProtocols: ['http', 'https', 'mailto'],
		allowedTags: basicMarkdownAllowedTags,
		tagFallback: (element) => {
			const text = element
				.slice(2)
				.filter((child): child is string => typeof child === 'string')
				.join('')
			return text ? text : false
		},
	}),
]

export function modrinthForceLinkTarget(target: string) {
	return defineComarkPlugin(() => ({
		name: 'modrinth-force-link-target',
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

function buildBasicRenderer(target?: string) {
	return createHtmlRenderer({
		linkify: true,
		unwrap: 'p',
		registerDefaultPlugins: false,
		plugins: target ? [...modrinthBasicPlugins, modrinthForceLinkTarget(target)] : modrinthBasicPlugins,
	})
}

const defaultBasicRenderer = buildBasicRenderer()

export const renderBasicInlineMarkdown = (
	markdown: string,
	options: {
		target?: string
	} = {},
): Promise<string> => {
	if (!options.target) return defaultBasicRenderer(markdown)
	return buildBasicRenderer(options.target)(markdown)
}
