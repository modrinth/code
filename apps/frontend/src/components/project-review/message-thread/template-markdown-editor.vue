<template>
	<MarkdownEditor
		ref="markdownEditor"
		v-bind="$attrs"
		:model-value="modelValue"
		:disabled="disabled"
		:extensions="extensions"
		@update:model-value="emit('update:modelValue', $event)"
		@ready="editorView = $event"
	>
		<template #after-preview="{ previewMode: preview }">
			<div class="ml-auto">
				<MarkdownTemplatePicker
					v-if="templates.length"
					ref="templatePicker"
					:templates="templates"
					:inline-query="templateQuery"
					:disabled="disabled || preview"
					@insert="insertTemplate"
					@dismiss="templateQuery = null"
				/>
			</div>
			<slot name="after-preview" :preview-mode="preview" />
		</template>
		<template #empty-preview><slot name="empty-preview" /></template>
	</MarkdownEditor>
</template>

<script setup lang="ts">
import { isolateHistory } from '@codemirror/commands'
import { syntaxTree } from '@codemirror/language'
import { EditorView } from '@codemirror/view'
import { MarkdownEditor } from '@modrinth/ui'
import { nextTick, onBeforeUnmount, ref, shallowRef, watch } from 'vue'

import MarkdownTemplatePicker from './markdown-template-picker.vue'
import type { MarkdownTemplate, MarkdownTemplateQuery } from './markdown-templates'

defineOptions({ inheritAttrs: false })
const props = defineProps<{
	modelValue: string
	templates: MarkdownTemplate[]
	disabled?: boolean
}>()
const emit = defineEmits<{ 'update:modelValue': [value: string] }>()
const markdownEditor = ref<InstanceType<typeof MarkdownEditor>>()
const editorView = shallowRef<EditorView>()
const templatePicker = ref<InstanceType<typeof MarkdownTemplatePicker>>()
const templateQuery = ref<MarkdownTemplateQuery | null>(null)
const extensions = [
	EditorView.domEventHandlers({
		keydown: (event) => templatePicker.value?.handleKeydown(event) ?? false,
	}),
	EditorView.updateListener.of((update) => {
		if (
			update.transactions.some((transaction) => transaction.isUserEvent('input.type')) ||
			(templateQuery.value &&
				(update.docChanged ||
					update.selectionSet ||
					update.geometryChanged ||
					update.viewportChanged))
		) {
			void updateTemplateQuery(update.view)
		}
	}),
]
async function updateTemplateQuery(view: EditorView) {
	await nextTick()
	if (view !== editorView.value) return
	if (!props.templates?.length || props.disabled || templatePicker.value?.disabled) {
		templateQuery.value = null
		return
	}
	const { head, empty } = view.state.selection.main
	const line = view.state.doc.lineAt(head)
	const match = /(?:^|\s)\/([^/\s]*)$/.exec(view.state.doc.sliceString(line.from, head))
	let node = syntaxTree(view.state).resolveInner(head, -1)
	while (node) {
		if (['FencedCode', 'CodeBlock', 'InlineCode', 'Link', 'URL', 'Autolink'].includes(node.name)) {
			templateQuery.value = null
			return
		}
		if (!node.parent) break
		node = node.parent
	}
	const rect = view.coordsAtPos(head)
	if (!empty || !match || !rect) {
		templateQuery.value = null
		return
	}
	templateQuery.value = {
		from: head - match[1].length - 1,
		to: head,
		query: match[1],
		anchor: {
			contextElement: view.contentDOM,
			getBoundingClientRect: () => {
				const caret =
					editorView.value === view
						? (view.coordsAtPos(view.state.selection.main.head) ?? rect)
						: rect
				return {
					left: caret.left,
					right: caret.right,
					top: caret.top,
					bottom: caret.bottom,
					x: caret.left,
					y: caret.top,
					width: caret.right - caret.left,
					height: caret.bottom - caret.top,
				}
			},
		},
	}
}

async function insertTemplate(body: string) {
	const view = editorView.value
	if (!view || props.disabled) return
	const doc = view.state.doc
	const from = templateQuery.value?.from ?? view.state.selection.main.head
	const to = templateQuery.value?.to ?? from
	const before = doc.sliceString(0, from)
	const after = doc.sliceString(to)
	const prefix = before.length
		? before.endsWith('\n\n')
			? ''
			: before.endsWith('\n')
				? '\n'
				: '\n\n'
		: ''
	const suffix = after.length
		? after.startsWith('\n\n')
			? ''
			: after.startsWith('\n')
				? '\n'
				: '\n\n'
		: ''
	const insert = prefix + body.trim() + suffix
	templateQuery.value = null
	view.dispatch({
		changes: { from, to, insert },
		selection: { anchor: from + insert.length },
		scrollIntoView: true,
		userEvent: 'input.template',
		annotations: isolateHistory.of('full'),
	})
	await focus()
}

watch(
	() => templatePicker.value?.activeDescendant,
	(value) => {
		const view = editorView.value
		if (!view) return
		if (value && templateQuery.value) {
			view.contentDOM.setAttribute('aria-controls', templatePicker.value!.listId)
			view.contentDOM.setAttribute('aria-activedescendant', value)
		} else {
			view.contentDOM.removeAttribute('aria-controls')
			view.contentDOM.removeAttribute('aria-activedescendant')
		}
	},
)

watch(
	() => templatePicker.value?.disabled,
	(disabled) => {
		if (disabled) templateQuery.value = null
	},
)
onBeforeUnmount(() => {
	editorView.value = undefined
})

async function focus() {
	await markdownEditor.value?.focus()
}
defineExpose({ focus })
</script>
