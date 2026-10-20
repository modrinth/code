import { type LanguageSupport, StreamLanguage, type StreamParser } from '@codemirror/language'

type LanguageLoader = () => Promise<LanguageSupport | StreamLanguage<unknown>>

function legacy(load: () => Promise<StreamParser<unknown>>): LanguageLoader {
	return async () => StreamLanguage.define(await load())
}

const clike = () => import('@codemirror/legacy-modes/mode/clike')
const xml = legacy(async () => (await import('@codemirror/legacy-modes/mode/xml')).xml)
const shell = legacy(async () => (await import('@codemirror/legacy-modes/mode/shell')).shell)
const properties = legacy(
	async () => (await import('@codemirror/legacy-modes/mode/properties')).properties,
)
const toml = legacy(async () => (await import('@codemirror/legacy-modes/mode/toml')).toml)

/**
 * Each language an editor can highlight, loaded only once used. Keyed by the names files are given
 * (see `getEditorLanguage`), which covers both the editor's own names and highlight.js's.
 */
const LANGUAGES: Record<string, LanguageLoader> = {
	json: async () => (await import('@codemirror/lang-json')).json(),
	javascript: async () => (await import('@codemirror/lang-javascript')).javascript(),
	typescript: async () =>
		(await import('@codemirror/lang-javascript')).javascript({ typescript: true }),
	html: async () => (await import('@codemirror/lang-html')).html(),
	css: async () => (await import('@codemirror/lang-css')).css(),
	markdown: async () => (await import('@codemirror/lang-markdown')).markdown(),
	yaml: async () => (await import('@codemirror/lang-yaml')).yaml(),
	java: async () => (await import('@codemirror/lang-java')).java(),
	kotlin: legacy(async () => (await clike()).kotlin),
	scala: legacy(async () => (await clike()).scala),
	c: legacy(async () => (await clike()).c),
	cpp: legacy(async () => (await clike()).cpp),
	c_cpp: legacy(async () => (await clike()).cpp),
	groovy: legacy(async () => (await import('@codemirror/legacy-modes/mode/groovy')).groovy),
	gradle: legacy(async () => (await import('@codemirror/legacy-modes/mode/groovy')).groovy),
	rust: legacy(async () => (await import('@codemirror/legacy-modes/mode/rust')).rust),
	go: legacy(async () => (await import('@codemirror/legacy-modes/mode/go')).go),
	golang: legacy(async () => (await import('@codemirror/legacy-modes/mode/go')).go),
	python: legacy(async () => (await import('@codemirror/legacy-modes/mode/python')).python),
	ruby: legacy(async () => (await import('@codemirror/legacy-modes/mode/ruby')).ruby),
	lua: legacy(async () => (await import('@codemirror/legacy-modes/mode/lua')).lua),
	powershell: legacy(
		async () => (await import('@codemirror/legacy-modes/mode/powershell')).powerShell,
	),
	dockerfile: legacy(
		async () => (await import('@codemirror/legacy-modes/mode/dockerfile')).dockerFile,
	),
	xml,
	sh: shell,
	bash: shell,
	toml,
	ini: properties,
	properties,
	mclog: legacy(async () => (await import('./mode-mclog')).mclog),
	mcfunction: legacy(async () => (await import('./mode-mcfunction')).mcfunction),
}

/** The highlighting for a language, or `null` when it's shown as plain text. */
export async function loadCodeLanguage(name: string) {
	return (await LANGUAGES[name]?.()) ?? null
}
