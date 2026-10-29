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
			:style="{ height: '20rem', fontSize: '0.875rem' }"
			class="ace-modrinth"
			@init="onEditorInit"
			@update:value="model = $event"
		/>
		<div
			v-else-if="loadFailed"
			role="alert"
			class="flex h-80 flex-col items-center justify-center gap-3 bg-bg-raised"
		>
			<p class="m-0 text-red">The Rhai editor could not be loaded.</p>
			<Button @click="loadEditor">Try again</Button>
		</div>
		<div
			v-else
			role="status"
			class="flex h-80 items-center justify-center gap-2 bg-bg-raised text-secondary"
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
import { computed, onMounted, ref, shallowRef, type Component } from 'vue'

const model = defineModel<string>({ required: true })
const props = defineProps<{ id: string; disabled?: boolean }>()
const editorComponent = shallowRef<Component | null>(null)
const languageOptions = shallowRef<Partial<Ace.EditorOptions>>({})
const loadFailed = ref(false)
const editorOptions = computed(() => ({
	...languageOptions.value,
	readOnly: props.disabled ?? false,
}))

async function loadEditor() {
	loadFailed.value = false
	try {
		await import('@modrinth/ui/src/utils/ace-theme')
		const [{ VAceEditor }, { RHAI_EDITOR_OPTIONS }] = await Promise.all([
			import('vue3-ace-editor'),
			import('./ace-rhai'),
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
}

onMounted(loadEditor)
</script>
