<script setup lang="ts">
import { Button, InlineValidationMessage } from '@modrinth/ui'
import { computed, ref } from 'vue'

import type { ArchiveInput } from '../types'
import { validateArchive } from '../validation'
import JsonEditor from './JsonEditor.vue'

const json = defineModel<string>({ required: true })
const props = defineProps<{
	editingId?: string
	persist: (entries: ArchiveInput[]) => Promise<boolean>
	saving: boolean
	saveError: string
}>()
const emit = defineEmits<{ saved: []; cancel: [] }>()
const submitted = ref(false)
const parsedJson = computed(() => {
	try {
		const archive: unknown = JSON.parse(json.value)
		const errors = validateArchive(archive)
		if (errors.length) return { archive: null, error: errors.join(' ') }
		const input = archive as ArchiveInput
		if (input.id !== props.editingId) {
			return {
				archive: null,
				error: props.editingId
					? 'The archive ID must remain unchanged.'
					: 'Omit the ID when adding a new archive.',
			}
		}
		return { archive: input, error: '' }
	} catch {
		return { archive: null, error: 'Enter valid JSON for a single archive object.' }
	}
})

async function save() {
	submitted.value = true
	if (props.saving) return
	if (parsedJson.value.archive && (await props.persist([parsedJson.value.archive]))) emit('saved')
}
</script>

<template>
	<div class="flex min-w-0 flex-col gap-3">
		<span class="font-semibold">Archive JSON</span>
		<JsonEditor
			v-model="json"
			label="Archive JSON"
			:disabled="saving"
			:error="submitted && !!parsedJson.error"
		/>
		<InlineValidationMessage :message="submitted ? parsedJson.error || saveError : undefined" />
		<div class="flex flex-wrap items-center gap-2">
			<slot name="actions-start" />
			<div class="ml-auto flex gap-2">
				<Button type="quiet" :disabled="saving" @click="emit('cancel')">Cancel</Button>
				<Button type="colored" color="brand" :disabled="saving" @click="save">
					{{ saving ? 'Saving...' : editingId ? 'Save changes' : 'Add archive' }}
				</Button>
			</div>
		</div>
	</div>
</template>
