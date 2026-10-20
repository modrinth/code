import { useLocalStorage } from '@vueuse/core'
import {
	effectScope,
	hasInjectionContext,
	inject,
	type InjectionKey,
	type Ref,
	ref,
	watch,
} from 'vue'

import { HIGHLIGHT_JS_THEMES } from '../utils/code-highlight-themes'

/** Modrinth's own colours, which code is highlighted with until another theme is picked. */
export const DEFAULT_CODE_THEME = 'modrinth'

const STYLE_ID = 'code-highlight-theme'
/** The stylesheet a server rendered page carries, replaced once the browser applies the theme. */
export const SERVER_CODE_THEME_STYLE_ID = 'code-highlight-theme-ssr'

/**
 * Where the theme is kept, when the host keeps it somewhere of its own choosing, e.g. a cookie
 * the server can read too, in place of local storage. Provided app-wide (`app.provide`).
 */
export const CODE_HIGHLIGHT_THEME_STORAGE: InjectionKey<Ref<string>> = Symbol.for(
	'modrinth:codeHighlightThemeStorage',
)

/** The stylesheet of a highlight.js theme, or `null` for Modrinth's own (or an unknown one). */
export async function loadCodeHighlightThemeCss(id: string): Promise<string | null> {
	const entry = HIGHLIGHT_JS_THEMES.find((candidate) => candidate.id === id)
	return entry ? entry.load().catch(() => null) : null
}

/**
 * Token colours read off the active theme into CSS variables (`--code-{name}`), for editors that
 * don't use highlight.js's classes (e.g. Ace), by the classes highlight.js gives each token.
 */
const TOKEN_CLASSES = {
	comment: 'hljs-comment',
	keyword: 'hljs-keyword',
	string: 'hljs-string',
	number: 'hljs-number',
	literal: 'hljs-literal',
	regexp: 'hljs-regexp',
	type: 'hljs-type',
	'built-in': 'hljs-built_in',
	function: 'hljs-title function_',
	class: 'hljs-title class_',
	title: 'hljs-title',
	variable: 'hljs-variable',
	attribute: 'hljs-attr',
	tag: 'hljs-name',
	meta: 'hljs-meta',
	punctuation: 'hljs-punctuation',
} as const

function isTransparent(color: string) {
	return color === 'transparent' || /^rgba\(.*,\s*0\)$/.test(color)
}

function clearTokenColors(root: HTMLElement) {
	for (const name of ['background', 'foreground', ...Object.keys(TOKEN_CLASSES)]) {
		root.style.removeProperty(`--code-${name}`)
	}
}

/** Renders a hidden sample of each token with the theme applied, and keeps the colours it gets. */
function readTokenColors(root: HTMLElement) {
	const sample = document.createElement('pre')
	sample.className = 'hljs'
	sample.style.cssText = 'position:absolute;visibility:hidden;pointer-events:none;left:-9999px'
	const tokens = Object.entries(TOKEN_CLASSES).map(([name, className]) => {
		const token = document.createElement('span')
		token.className = className
		token.textContent = name
		sample.appendChild(token)
		return [name, token] as const
	})
	document.body.appendChild(sample)

	const style = getComputedStyle(sample)
	if (!isTransparent(style.backgroundColor)) {
		root.style.setProperty('--code-background', style.backgroundColor)
	}
	root.style.setProperty('--code-foreground', style.color)
	for (const [name, token] of tokens) {
		root.style.setProperty(`--code-${name}`, getComputedStyle(token).color)
	}
	sample.remove()
}

function createCodeHighlightTheme(theme: Ref<string>) {
	let started = false

	async function apply(id: string) {
		const root = document.documentElement
		if (!HIGHLIGHT_JS_THEMES.some((candidate) => candidate.id === id)) {
			document.getElementById(STYLE_ID)?.remove()
			document.getElementById(SERVER_CODE_THEME_STYLE_ID)?.remove()
			delete root.dataset.codeTheme
			clearTokenColors(root)
			return
		}

		const css = await loadCodeHighlightThemeCss(id)
		if (theme.value !== id) return
		if (css == null) {
			theme.value = DEFAULT_CODE_THEME
			return
		}

		let style = document.getElementById(STYLE_ID) as HTMLStyleElement | null
		if (!style) {
			style = document.createElement('style')
			style.id = STYLE_ID
			document.head.appendChild(style)
		}
		style.textContent = css
		document.getElementById(SERVER_CODE_THEME_STYLE_ID)?.remove()
		root.dataset.codeTheme = id
		clearTokenColors(root)
		readTokenColors(root)
	}

	/** Applies the theme now and whenever it changes. Only runs in the browser, once. */
	function start() {
		if (started || typeof document === 'undefined') return
		started = true
		watch(theme, (id) => void apply(id), { immediate: true })
	}

	return { theme, themes: HIGHLIGHT_JS_THEMES, start }
}

let browserState: ReturnType<typeof createCodeHighlightTheme> | undefined

/**
 * The theme code is highlighted with everywhere, kept per device (in local storage, unless the
 * host provides `CODE_HIGHLIGHT_THEME_STORAGE`). Once started (by the app's root), a picked
 * highlight.js theme is loaded and applied to every highlighted block, and its colours are passed
 * on to editors with highlighting of their own; `modrinth` keeps the defaults. Shared by the whole
 * page in the browser, while each server render gets its own, from its own request.
 */
export function useCodeHighlightTheme() {
	const provided = hasInjectionContext() ? inject(CODE_HIGHLIGHT_THEME_STORAGE, null) : null
	if (typeof window === 'undefined') {
		return createCodeHighlightTheme(provided ?? ref(DEFAULT_CODE_THEME))
	}
	browserState ??= effectScope(true).run(() =>
		createCodeHighlightTheme(
			provided ?? useLocalStorage('code-highlight-theme', DEFAULT_CODE_THEME),
		),
	)!
	return browserState
}
