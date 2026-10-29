<script setup lang="ts">
import { Button, InlineValidationMessage, NewModal } from '@modrinth/ui'
import { computed, ref } from 'vue'

import type { ArchiveInput } from '../types'
import { parseImport } from '../validation'
import JsonEditor from './JsonEditor.vue'

const props = defineProps<{
	existingIds: string[]
	persist: (entries: ArchiveInput[]) => Promise<boolean>
	saving: boolean
	saveError: string
}>()
const submitted = ref(false)
const modal = ref<InstanceType<typeof NewModal>>()
const json = ref('')
const preview = computed(() => {
	if (!json.value.trim()) return { entries: [], error: '' }
	try {
		return { entries: parseImport(json.value, new Set(props.existingIds)), error: '' }
	} catch (error) {
		return { entries: [], error: error instanceof Error ? error.message : String(error) }
	}
})
const replacements = computed(() => preview.value.entries.filter((entry) => entry.id).length)
function show() {
	json.value = ''
	submitted.value = false
	modal.value?.show()
}
async function save() {
	submitted.value = true
	if (!preview.value.entries.length || preview.value.error || props.saving) return
	if (await props.persist(preview.value.entries)) modal.value?.hide()
}
defineExpose({ show })
</script>

<template>
	<NewModal
		ref="modal"
		header="Bulk import copyright archives"
		width="720px"
		max-width="calc(100vw - 2rem)"
		:close-on-click-outside="false"
		:disable-close="saving"
	>
		<div class="flex flex-col gap-4">
			<p class="m-0">
				Paste a JSON array of archives. Omit IDs to create new archives. Include an existing ID to
				replace its entire record.
			</p>
			<div class="flex flex-col gap-2">
				<span class="font-semibold">Archives JSON</span>
				<JsonEditor
					v-model="json"
					label="Archives JSON"
					:disabled="saving"
					:error="submitted && !!preview.error"
				/>
			</div>
			<InlineValidationMessage
				:message="
					submitted
						? preview.error ||
							(!preview.entries.length ? 'Enter a JSON array of archives.' : saveError)
						: undefined
				"
			/>
			<p v-if="preview.entries.length" role="status" class="m-0">
				{{ preview.entries.length - replacements }} to create, {{ replacements }} to replace.
			</p>
		</div>
		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="quiet" :disabled="saving" @click="modal?.hide()">Cancel</Button>
				<Button type="colored" color="brand" :disabled="saving" @click="save">{{
					saving ? 'Importing...' : 'Import archives'
				}}</Button>
			</div>
		</template>
	</NewModal>
</template>
