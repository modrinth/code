<template>
	<div ref="host" class="code-editor font-mono" />
</template>

<script setup lang="ts">
import { acceptCompletion, closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete'
import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands'
import {
	bracketMatching,
	indentOnInput,
	indentUnit,
	syntaxHighlighting,
} from '@codemirror/language'
import { highlightSelectionMatches } from '@codemirror/search'
import { Compartment, EditorState, type Extension, Prec } from '@codemirror/state'
import {
	drawSelection,
	EditorView,
	highlightActiveLine,
	highlightActiveLineGutter,
	highlightSpecialChars,
	keymap,
	lineNumbers,
	tooltips,
} from '@codemirror/view'
import { onBeforeUnmount, onMounted, shallowRef, useTemplateRef, watch } from 'vue'

import { findWithoutPanel } from '#ui/utils/codemirror/find'
import { highlightJsHighlighting } from '#ui/utils/codemirror/highlight-js'
import { highlightJsClasses } from '#ui/utils/codemirror/highlight-js-classes'
import { loadCodeLanguage } from '#ui/utils/codemirror/languages'

/**
 * A code editor, coloured by the code highlighting theme the user picked (see
 * `useCodeHighlightTheme`). It fills the size it's given.
 */
const props = withDefaults(
	defineProps<{
		modelValue: string
		/** The language to highlight, by the names `getEditorLanguage` gives files. */
		language?: string
		readonly?: boolean
		/** Wrap long lines instead of scrolling sideways. */
		lineWrapping?: boolean
		/** What one level of indentation is, e.g. a tab or two spaces. */
		indent?: string
		tabSize?: number
		/** Further extensions, e.g. key bindings or completions of the page's own. */
		extensions?: Extension[]
	}>(),
	{
		language: 'text',
		readonly: false,
		lineWrapping: false,
		indent: '\t',
		tabSize: 4,
		extensions: () => [],
	},
)

const emit = defineEmits<{
	'update:modelValue': [value: string]
	save: []
}>()

const host = useTemplateRef<HTMLDivElement>('host')
const view = shallowRef<EditorView | null>(null)

const languageCompartment = new Compartment()
const highlightingCompartment = new Compartment()
const readonlyCompartment = new Compartment()
const wrappingCompartment = new Compartment()
const indentCompartment = new Compartment()
const extraCompartment = new Compartment()

function indentation() {
	return [indentUnit.of(props.indent), EditorState.tabSize.of(props.tabSize)]
}

/** Popups such as completions, styled like the site's menus as they're shown outside the editor. */
const tooltipTheme = EditorView.theme({
	'.cm-tooltip': {
		background: 'var(--surface-3)',
		color: 'var(--color-text-default)',
		border: '1px solid var(--surface-5)',
		borderRadius: '0.5rem',
		overflow: 'hidden',
	},
	'.cm-tooltip.cm-tooltip-autocomplete > ul': {
		fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
	},
	'.cm-tooltip.cm-tooltip-autocomplete > ul > li[aria-selected]': {
		background: 'var(--color-brand-highlight)',
		color: 'var(--color-contrast)',
	},
	'.cm-completionDetail': {
		color: 'var(--color-text-tertiary)',
	},
})

/**
 * Highlights as highlight.js does where it knows the language, matching code blocks elsewhere, or
 * else from the language's own parser, given the same classes.
 */
function highlighting(name: string, editor: EditorView) {
	return highlightJsHighlighting(name, editor.state.doc) ?? syntaxHighlighting(highlightJsClasses)
}

let languageRequest = 0
async function applyLanguage(name: string) {
	const request = ++languageRequest
	const language = await loadCodeLanguage(name).catch(() => null)
	const editor = view.value
	if (request !== languageRequest || !editor) return
	editor.dispatch({
		effects: [
			languageCompartment.reconfigure(language ?? []),
			highlightingCompartment.reconfigure(highlighting(name, editor)),
		],
	})
}

/** Moves the editor over to the document it's now shown in, e.g. once its panel pops out. */
function followDocument() {
	const editor = view.value
	const root = host.value?.getRootNode()
	if (
		editor &&
		root &&
		root !== editor.root &&
		(root instanceof Document || root instanceof ShadowRoot)
	) {
		editor.setRoot(root)
	}
}

let resizeObserver: ResizeObserver | null = null

onMounted(() => {
	view.value = new EditorView({
		parent: host.value!,
		state: EditorState.create({
			doc: props.modelValue,
			extensions: [
				lineNumbers(),
				highlightActiveLineGutter(),
				highlightSpecialChars(),
				history(),
				drawSelection(),
				indentOnInput(),
				bracketMatching(),
				closeBrackets(),
				highlightActiveLine(),
				highlightSelectionMatches(),
				findWithoutPanel,
				EditorView.editorAttributes.of({ class: 'hljs' }),
				tooltips({ parent: host.value!.ownerDocument.body }),
				tooltipTheme,
				Prec.highest(
					keymap.of([
						{
							key: 'Mod-s',
							preventDefault: true,
							run: () => {
								emit('save')
								return true
							},
						},
					]),
				),
				keymap.of([
					{ key: 'Tab', run: acceptCompletion },
					...closeBracketsKeymap,
					...defaultKeymap,
					...historyKeymap,
					indentWithTab,
				]),
				EditorView.updateListener.of((update) => {
					if (update.docChanged) emit('update:modelValue', update.state.doc.toString())
				}),
				languageCompartment.of([]),
				highlightingCompartment.of([]),
				readonlyCompartment.of(EditorState.readOnly.of(props.readonly)),
				wrappingCompartment.of(props.lineWrapping ? EditorView.lineWrapping : []),
				indentCompartment.of(indentation()),
				extraCompartment.of(props.extensions),
			],
		}),
	})
	void applyLanguage(props.language)

	resizeObserver = new ResizeObserver(followDocument)
	resizeObserver.observe(host.value!)
})

onBeforeUnmount(() => {
	resizeObserver?.disconnect()
	view.value?.destroy()
	view.value = null
})

watch(
	() => props.modelValue,
	(value) => {
		const editor = view.value
		if (!editor || value === editor.state.doc.toString()) return
		editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: value } })
	},
)

watch(() => props.language, applyLanguage)

watch(
	() => props.readonly,
	(readonly) =>
		view.value?.dispatch({
			effects: readonlyCompartment.reconfigure(EditorState.readOnly.of(readonly)),
		}),
)

watch(
	() => props.lineWrapping,
	(wrapping) =>
		view.value?.dispatch({
			effects: wrappingCompartment.reconfigure(wrapping ? EditorView.lineWrapping : []),
		}),
)

watch([() => props.indent, () => props.tabSize], () =>
	view.value?.dispatch({ effects: indentCompartment.reconfigure(indentation()) }),
)

watch(
	() => props.extensions,
	(extensions) => view.value?.dispatch({ effects: extraCompartment.reconfigure(extensions) }),
)

function focus() {
	view.value?.focus()
}

defineExpose({ view, focus })
</script>

<style scoped>
.code-editor {
	position: relative;
	overflow: hidden;
}

.code-editor :deep(.cm-editor) {
	height: 100%;
	padding: 0;
	font-size: inherit;
}

html:not([data-code-theme]) .code-editor :deep(.cm-editor) {
	background: var(--surface-3);
	color: var(--color-text-default);
}

.code-editor :deep(.cm-editor.cm-focused) {
	outline: none;
}

.code-editor :deep(.cm-scroller) {
	font-family: inherit;
	line-height: 1.5;
	overflow: auto;
}

.code-editor :deep(.cm-content) {
	caret-color: currentColor;
}

.code-editor :deep(.cm-gutters) {
	background: inherit;
	color: color-mix(in srgb, currentColor 45%, transparent);
	border-right: 1px solid color-mix(in srgb, currentColor 15%, transparent);
}

.code-editor :deep(.cm-activeLine),
.code-editor :deep(.cm-activeLineGutter) {
	background: color-mix(in srgb, currentColor 7%, transparent);
}

.code-editor :deep(.cm-activeLineGutter) {
	color: currentColor;
}

.code-editor :deep(.cm-cursor) {
	border-left-color: currentColor;
}

.code-editor :deep(.cm-selectionBackground) {
	background: color-mix(in srgb, currentColor 18%, transparent) !important;
}

.code-editor :deep(.cm-focused .cm-selectionBackground) {
	background: color-mix(in srgb, currentColor 25%, transparent) !important;
}

.code-editor :deep(.cm-selectionMatch) {
	background: color-mix(in srgb, currentColor 12%, transparent);
}

.code-editor :deep(.cm-matchingBracket) {
	background: transparent;
	outline: 1px solid color-mix(in srgb, currentColor 40%, transparent);
}

.code-editor :deep(.cm-searchMatch) {
	background: color-mix(in srgb, var(--color-orange) 30%, transparent);
	outline: none;
}

.code-editor :deep(.cm-searchMatch-selected) {
	background: color-mix(in srgb, var(--color-orange) 60%, transparent);
}

.code-editor :deep(.cm-panels) {
	height: 0;
	overflow: hidden;
	border: none;
}
</style>
