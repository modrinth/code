import { renderHTML } from '@comark/html'
import { createParse } from 'comark'
import highlight from 'comark/plugins/highlight'
import { codeToHtml } from 'shiki'

import { modrinthMarkdownSecurity, type ModrinthMarkdownSecurityOptions } from './security'

export interface MarkdownRenderOptions extends ModrinthMarkdownSecurityOptions {
	highlightCode?: boolean
}

export type MarkdownRenderer = (markdown: string) => Promise<string>

const shikiThemes = {
	light: 'github-light',
	dark: 'github-dark',
}

function normalizeLanguage(language: string): string {
	const normalized = language.trim().toLowerCase()
	const aliases: Record<string, string> = {
		command: 'mcfunction',
		fxml: 'xml',
		htm: 'html',
		js: 'javascript',
		json5: 'json',
		kt: 'kotlin',
		kubejs: 'javascript',
		mcui: 'xml',
		py: 'python',
		sk: 'skript',
		xhtml: 'html',
		yml: 'yaml',
	}

	return aliases[normalized] ?? normalized
}

export function createMarkdownRenderer(options: MarkdownRenderOptions = {}): MarkdownRenderer {
	const plugins = []

	if (options.highlightCode) {
		plugins.push(
			highlight({
				preStyles: false,
				registerDefaultLanguages: true,
				// TODO: Add Shiki grammars for skript and mcfunction before re-enabling those legacy Highlight.js aliases.
			}),
		)
	}

	plugins.push(modrinthMarkdownSecurity(options))

	const parse = createParse({
		autoClose: false,
		html: true,
		plugins,
	})

	return async (markdown: string) => {
		if (!markdown) return ''

		const tree = await parse(markdown)
		return renderHTML(tree)
	}
}

const renderMarkdownDefault = createMarkdownRenderer()
const renderHighlightedMarkdownDefault = createMarkdownRenderer({ highlightCode: true })

export async function renderMarkdown(markdown: string): Promise<string> {
	return renderMarkdownDefault(markdown)
}

export async function renderHighlightedMarkdown(markdown: string): Promise<string> {
	return renderHighlightedMarkdownDefault(markdown)
}

export async function renderMarkdownInline(markdown: string): Promise<string> {
	const html = await renderMarkdown(markdown)
	const match = html.match(/^<p>([\s\S]*)<\/p>\s*$/)
	return match ? match[1] : html
}

function escapeHtml(value: string): string {
	return value
		.replaceAll('&', '&amp;')
		.replaceAll('<', '&lt;')
		.replaceAll('>', '&gt;')
		.replaceAll('"', '&quot;')
}

function plainCodeLines(code: string): string[] {
	return code.split('\n').map(escapeHtml)
}

export async function highlightCodeLines(code: string, language: string): Promise<string[]> {
	if (!code) return []

	try {
		const html = await codeToHtml(code, {
			lang: normalizeLanguage(language),
			themes: shikiThemes,
		})

		if (typeof DOMParser === 'undefined') return plainCodeLines(code)

		const document = new DOMParser().parseFromString(html, 'text/html')
		const lines = [...document.querySelectorAll('span.line')].map((line) => line.innerHTML)
		return lines.length > 0 ? lines : plainCodeLines(code)
	} catch {
		return plainCodeLines(code)
	}
}
