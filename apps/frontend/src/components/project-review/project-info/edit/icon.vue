<template>
	<div class="flex flex-col gap-2">
		<AiImageWarningModal ref="aiImageWarningModal" />
		<div class="flex items-center gap-3">
			<Avatar
				:src="deleted ? undefined : (previewImage ?? project?.icon_url ?? undefined)"
				:alt="project?.name"
				size="5rem"
				no-shadow
			/>
			<div class="flex flex-col gap-2">
				<FileButton
					:max-size="524288"
					accept="image/png,image/jpeg,image/gif,image/webp"
					:prompt="formatMessage(messages.uploadIcon)"
					:aria-label="formatMessage(messages.uploadIcon)"
					:disabled="saving"
					size="sm"
					@change="selectIcon"
				>
					<UploadIcon aria-hidden="true" />
				</FileButton>
				<Button
					v-if="!deleted && (icon || project?.icon_url)"
					size="sm"
					:disabled="saving"
					@click="removeIcon"
				>
					<TrashIcon aria-hidden="true" />
					{{ formatMessage(messages.removeIcon) }}
				</Button>
			</div>
		</div>
		<div class="flex justify-end gap-1">
			<Button type="quiet" size="xs" :disabled="saving" @click="emit('done')">
				{{ formatMessage(commonMessages.cancelButton) }}
			</Button>
			<Button
				type="colored"
				color="brand"
				size="xs"
				:loading="saving"
				:disabled="saving || (!icon && !deleted)"
				@click="save"
			>
				{{ formatMessage(commonMessages.saveButton) }}
			</Button>
		</div>
	</div>
</template>

<script setup lang="ts">
import { TrashIcon, UploadIcon } from '@modrinth/assets'
import { Avatar, Button, commonMessages, defineMessages, FileButton, useVIntl } from '@modrinth/ui'
import { useObjectUrl } from '@vueuse/core'
import { onScopeDispose, ref, useTemplateRef } from 'vue'

import AiImageWarningModal from '~/components/ui/AiImageWarningModal.vue'
import { fileDeclaresAi } from '~/helpers/c2pa'

import { useProjectInfoEdit } from './use-project-edit'

const emit = defineEmits<{ done: [] }>()
const messages = defineMessages({
	uploadIcon: {
		id: 'moderation.project-review.edit.upload-icon',
		defaultMessage: 'Upload icon',
	},
	removeIcon: {
		id: 'moderation.project-review.edit.remove-icon',
		defaultMessage: 'Remove icon',
	},
})
const { formatMessage } = useVIntl()
const { project, saving, beginEditing, saveIcon } = useProjectInfoEdit()
const icon = ref<File>()
const deleted = ref(false)
const previewImage = useObjectUrl(icon)
const aiImageWarningModal = useTemplateRef('aiImageWarningModal')
let selection = 0

beginEditing()
onScopeDispose(() => selection++)

async function selectIcon(files: File[]) {
	const file = files[0]
	if (!file || saving.value) return
	const currentSelection = ++selection
	const declaresAi = await fileDeclaresAi(file)
	if (currentSelection !== selection) return
	if (declaresAi) {
		aiImageWarningModal.value?.show()
		return
	}
	icon.value = file
	deleted.value = false
}

function removeIcon() {
	selection++
	icon.value = undefined
	deleted.value = true
}

async function save() {
	if (saving.value || (!icon.value && !deleted.value)) return
	selection++
	if (await saveIcon(deleted.value ? null : (icon.value ?? null))) emit('done')
}
</script>
