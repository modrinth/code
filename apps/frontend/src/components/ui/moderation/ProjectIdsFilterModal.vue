<script setup lang="ts">
import { CheckIcon, XIcon } from '@modrinth/assets'
import { Button, NewModal, Textarea } from '@modrinth/ui'
import { nextTick, ref, useTemplateRef } from 'vue'

const emit = defineEmits<{
	apply: [projectIds: string[]]
	cancel: []
}>()

const modalRef = useTemplateRef<InstanceType<typeof NewModal>>('modalRef')
const textareaRef = useTemplateRef<InstanceType<typeof Textarea>>('textareaRef')
const input = ref('')
const error = ref('')
let applied = false

function parseProjectIds(value: string): string[] {
	return [
		...new Set(
			value
				.split(/[,\r\n]+/)
				.map((id) => id.replace(/\s+/g, ''))
				.filter(Boolean),
		),
	]
}

async function show(projectIds: string[]) {
	input.value = projectIds.join('\n')
	error.value = ''
	applied = false
	modalRef.value?.show()
	await nextTick()
	textareaRef.value?.focus()
}

function hide() {
	modalRef.value?.hide()
}

function handleHide() {
	if (!applied) emit('cancel')
}

function apply() {
	const projectIds = parseProjectIds(input.value)
	if (projectIds.length === 0) {
		error.value = 'Enter at least one project ID.'
		return
	}

	applied = true
	emit('apply', projectIds)
	hide()
}

defineExpose({ show, hide })
</script>

<template>
	<NewModal
		ref="modalRef"
		header="Filter by project IDs"
		width="36rem"
		max-width="calc(100vw - 2rem)"
		:on-hide="handleHide"
	>
		<form class="flex flex-col gap-4" @submit.prevent="apply">
			<div class="flex flex-col gap-2">
				<label class="font-semibold text-contrast" for="moderation-project-ids">
					Project IDs
				</label>
				<Textarea
					id="moderation-project-ids"
					ref="textareaRef"
					v-model="input"
					:rows="10"
					:error="!!error"
					resize="vertical"
					placeholder="Enter project IDs separated by commas or new lines"
					wrapper-class="min-h-48"
					@input="error = ''"
				/>
				<span v-if="error" class="text-sm font-semibold text-red">{{ error }}</span>
				<span v-else class="text-sm text-secondary">
					Separate IDs with commas or new lines. Whitespace and duplicate IDs are removed.
				</span>
			</div>

			<div class="flex justify-end gap-2">
				<Button native-type="button" @click="hide">
					<XIcon aria-hidden="true" />
					Cancel
				</Button>
				<Button type="colored" color="brand" native-type="submit">
					<CheckIcon aria-hidden="true" />
					Apply
				</Button>
			</div>
		</form>
	</NewModal>
</template>
