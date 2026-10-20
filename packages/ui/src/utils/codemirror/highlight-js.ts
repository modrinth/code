import type { Range, Text } from '@codemirror/state'
import { Decoration, type DecorationSet, ViewPlugin, type ViewUpdate } from '@codemirror/view'
import { hljs } from '@modrinth/utils/highlightjs/index'

/** The highlight.js grammar for each editor language that has one (see `getEditorLanguage`). */
const HIGHLIGHT_JS_LANGUAGES: Record<string, string> = {
	java: 'java',
	kotlin: 'kotlin',
	scala: 'scala',
	groovy: 'groovy',
	gradle: 'gradle',
	json: 'json',
	yaml: 'yaml',
	xml: 'xml',
	html: 'xml',
	properties: 'properties',
	ini: 'ini',
	toml: 'ini',
	javascript: 'javascript',
	typescript: 'javascript',
	python: 'python',
	lua: 'lua',
	mcfunction: 'mcfunction',
}

/** Past this size, highlighting the whole document on each change would make typing lag. */
const MAX_LENGTH = 512 * 1024

const ENTITIES: Record<string, string> = {
	'&amp;': '&',
	'&lt;': '<',
	'&gt;': '>',
	'&quot;': '"',
	'&#x27;': "'",
}

const TOKENS = /<span class="([^"]*)">|<\/span>|&(?:amp|lt|gt|quot|#x27);|[^<&]+/g

const marks = new Map<string, Decoration>()
function markFor(className: string) {
	let mark = marks.get(className)
	if (!mark) {
		mark = Decoration.mark({ class: className })
		marks.set(className, mark)
	}
	return mark
}

/** Marks each token highlight.js finds with the classes it gives it, from its HTML output. */
function decorate(doc: Text, language: string): DecorationSet {
	let html: string
	try {
		html = hljs.highlight(doc.toString(), { language, ignoreIllegals: true }).value
	} catch {
		return Decoration.none
	}

	const ranges: Range<Decoration>[] = []
	const open: { className: string; from: number }[] = []
	let position = 0
	for (const [token, className] of html.matchAll(TOKENS)) {
		if (className !== undefined) {
			open.push({ className, from: position })
		} else if (token === '</span>') {
			const span = open.pop()
			if (span && span.from < position)
				ranges.push(markFor(span.className).range(span.from, position))
		} else {
			position += ENTITIES[token]?.length ?? token.length
		}
	}
	return Decoration.set(ranges, true)
}

/**
 * Highlights the document as highlight.js does, so code is coloured exactly like highlighted code
 * blocks elsewhere (e.g. technical review), or `null` for languages it has no grammar for.
 */
export function highlightJsHighlighting(name: string, doc: Text) {
	const language = HIGHLIGHT_JS_LANGUAGES[name]
	if (!language || !hljs.getLanguage(language) || doc.length > MAX_LENGTH) return null

	return ViewPlugin.fromClass(
		class {
			decorations: DecorationSet

			constructor(view: { state: { doc: Text } }) {
				this.decorations = decorate(view.state.doc, language)
			}

			update(update: ViewUpdate) {
				if (update.docChanged) this.decorations = decorate(update.state.doc, language)
			}
		},
		{ decorations: (plugin) => plugin.decorations },
	)
}
