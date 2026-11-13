<template>
	<div class="overflow-hidden rounded-[20px] border border-solid border-surface-4">
		<component
			:is="editorComponent"
			v-if="editorComponent"
			:value="model"
			lang="rhai"
			theme="modrinth"
			:print-margin="false"
			:options="editorOptions"
			:style="{ fontSize: '0.875rem' }"
			class="ace-modrinth h-full"
			@init="onEditorInit"
			@update:value="model = $event"
		/>
		<div
			v-else-if="loadFailed"
			role="alert"
			class="flex h-full flex-col items-center justify-center gap-3 bg-surface-1"
		>
			<p class="m-0 text-red">The Rhai editor could not be loaded.</p>
			<Button @click="loadEditor">Try again</Button>
		</div>
		<div
			v-else
			role="status"
			class="flex h-full items-center justify-center gap-2 bg-surface-1 text-secondary"
		>
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
const props = defineProps<{
	id: string
	disabled?: boolean
	errorId?: string
}>()
const editorComponent = shallowRef<Component | null>(null)
const languageOptions = shallowRef<Partial<Ace.EditorOptions>>({})
const loadFailed = ref(false)
const editorInput = shallowRef<HTMLTextAreaElement>()
const editorOptions = computed(() => ({
	...languageOptions.value,
	enableKeyboardAccessibility: true,
	readOnly: props.disabled ?? false,
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
	editor.session.setUseWrapMode(true)
	const input = editor.textInput.getElement()
	input.id = props.id
	input.setAttribute('aria-label', 'Rhai expression')
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
