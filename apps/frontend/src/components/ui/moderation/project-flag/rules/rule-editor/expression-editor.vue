<template>
	<div
		class="overflow-hidden"
		:class="{
			'rounded-[20px] border border-solid border-surface-4': !frameless,
			'source-editor bg-surface-2': variant === 'source',
			'bg-surface-1': variant !== 'source',
		}"
		:style="{ height }"
	>
		<component
			:is="editorComponent"
			v-if="editorComponent"
			:value="model"
			lang="rhai"
			theme="modrinth"
			:print-margin="false"
			:options="editorOptions"
			:style="{ fontSize: variant === 'source' ? '13px' : '0.875rem' }"
			class="ace-modrinth h-full"
			@init="onEditorInit"
			@update:value="model = $event"
		/>
		<div
			v-else-if="loadFailed"
			role="alert"
			class="flex h-full flex-col items-center justify-center gap-3"
		>
			<p class="m-0 text-red">The Rhai editor could not be loaded.</p>
			<Button @click="loadEditor">Try again</Button>
		</div>
		<div v-else role="status" class="flex h-full items-center justify-center gap-2 text-secondary">
			<LoaderCircleIcon class="animate-spin" />
			Loading Rhai editor…
		</div>
	</div>
</template>

<script setup lang="ts">
import { LoaderCircleIcon } from '@modrinth/assets'
import { Button } from '@modrinth/ui'
import type { Ace } from 'ace-builds'
import { type Component, computed, onMounted, ref, shallowRef, watchEffect } from 'vue'

const model = defineModel<string>({ required: true })
const props = withDefaults(
	defineProps<{
		id: string
		disabled?: boolean
		errorId?: string
		frameless?: boolean
		height?: string
		label?: string
		variant?: 'default' | 'source'
		wordWrap?: boolean
	}>(),
	{ wordWrap: true, label: 'Rhai expression' },
)
const editorComponent = shallowRef<Component | null>(null)
const languageOptions = shallowRef<Partial<Ace.EditorOptions>>({})
const loadFailed = ref(false)
const editorInput = shallowRef<HTMLTextAreaElement>()
const editorOptions = computed(() => ({
	...languageOptions.value,
	enableKeyboardAccessibility: true,
	readOnly: props.disabled ?? false,
	wrap: props.wordWrap,
}))

async function loadEditor() {
	loadFailed.value = false
	try {
		await import('@modrinth/ui/src/utils/ace-theme')
		const [{ VAceEditor }, { RHAI_EDITOR_OPTIONS }] = await Promise.all([
			import('vue3-ace-editor'),
			import('../ace-rhai'),
			import('ace-builds/src-noconflict/ext-language_tools'),
		])
		languageOptions.value = RHAI_EDITOR_OPTIONS
		editorComponent.value = VAceEditor
	} catch {
		loadFailed.value = true
	}
}

function onEditorInit(editor: Ace.Editor) {
	if (props.variant === 'source') {
		editor.renderer.setPadding(16)
		editor.renderer.setScrollMargin(12, 12)
	}
	const input = editor.textInput.getElement()
	input.id = props.id
	input.setAttribute('aria-label', props.label)
	editorInput.value = input
}

watchEffect(() => {
	const input = editorInput.value
	if (!input) return
	if (props.errorId) {
		input.setAttribute('aria-describedby', props.errorId)
		input.setAttribute('aria-invalid', 'true')
	} else {
		input.removeAttribute('aria-describedby')
		input.removeAttribute('aria-invalid')
	}
})

onMounted(loadEditor)
</script>

<style scoped>
.source-editor :deep(.ace-modrinth) {
	background: var(--surface-2);
	line-height: 1.6;
}

.source-editor :deep(.ace_gutter) {
	border-right: 1px solid var(--surface-3);
	background: var(--surface-2);
	color: var(--color-text-secondary);
}

.source-editor :deep(.ace_gutter-cell) {
	padding-inline: 1rem;
}

.source-editor :deep(.ace_marker-layer .ace_active-line),
.source-editor :deep(.ace_gutter-active-line) {
	background: var(--surface-3);
}
</style>
