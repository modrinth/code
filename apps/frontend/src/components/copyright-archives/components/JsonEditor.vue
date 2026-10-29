<script setup lang="ts">
import { InlineValidationMessage } from '@modrinth/ui'
import type { Ace } from 'ace-builds'
import { type Component, onMounted, ref, shallowRef } from 'vue'

const json = defineModel<string>({ required: true })
defineProps<{ disabled: boolean; label: string; error?: boolean }>()
const editorComponent = shallowRef<Component | null>(null)
const loadError = ref('')
const options: Partial<Ace.EditorOptions> = {
	useWorker: false,
	tabSize: 4,
	useSoftTabs: false,
	showFoldWidgets: true,
	highlightActiveLine: true,
}

onMounted(async () => {
	try {
		const [{ VAceEditor }] = await Promise.all([
			import('vue3-ace-editor'),
			import('@modrinth/ui/src/utils/ace-theme'),
		])
		await import('ace-builds/src-noconflict/mode-json')
		editorComponent.value = VAceEditor
	} catch {
		loadError.value = 'Could not load the JSON editor. Reload the page to try again.'
	}
})
</script>

<template>
	<div
		class="overflow-hidden rounded-xl border border-solid border-surface-4"
		:class="{ '!border-red': error }"
	>
		<component
			:is="editorComponent"
			v-if="editorComponent"
			v-model:value="json"
			lang="json"
			theme="modrinth"
			:aria-label="label"
			:readonly="disabled"
			:print-margin="false"
			:options="options"
			:style="{ height: 'min(28rem, 55vh)', fontSize: '0.875rem' }"
			class="ace-modrinth"
		/>
		<InlineValidationMessage v-else-if="loadError" :message="loadError" class="p-4" />
		<div v-else class="flex h-64 items-center justify-center text-secondary" role="status">
			Loading JSON editor...
		</div>
	</div>
</template>
