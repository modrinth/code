<template>
	<NewModal ref="modal" :header="header" :closable="!uploadProgress">
		<div class="flex w-[496px] max-w-full flex-col gap-6">
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.fileLabel) }}</span>
				<ArchiveInput
					v-model="file"
					v-model:processing="processing"
					accept=".mrpack"
					:local-files="false"
					:disabled="!!uploadProgress"
				/>
			</div>
			<ModpackUploadProgress v-if="uploadProgress" :progress="uploadProgress" />
			<div class="flex flex-wrap items-center justify-end gap-2">
				<Button type="outlined" size="lg" :disabled="!!uploadProgress" @click="hide">
					<XIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="brand"
					size="lg"
					:disabled="!mrpack || processing || !!uploadProgress"
					@click="confirm"
				>
					<component :is="confirmIcon" aria-hidden="true" />
					{{ confirmLabel }}
				</Button>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import type { UploadProgress } from '@modrinth/api-client'
import { XIcon } from '@modrinth/assets'
import { type Component, computed, ref } from 'vue'

import { Button } from '#ui/components/base/buttons'
import ArchiveInput from '#ui/components/flows/creation-flow-modal/components/curseforge-stage/archive-input.vue'
import ModpackUploadProgress from '#ui/components/flows/creation-flow-modal/components/ModpackUploadProgress.vue'
import type { CurseForgePackFile } from '#ui/components/flows/creation-flow-modal/creation-flow-context'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

defineProps<{
	header: string
	confirmLabel: string
	confirmIcon: Component
	uploadProgress?: UploadProgress | null
}>()

const emit = defineEmits<{
	confirm: [file: File]
}>()

const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal>>()
const file = ref<CurseForgePackFile | null>(null)
const processing = ref(false)
const mrpack = computed(() => (file.value instanceof File ? file.value : null))

const messages = defineMessages({
	fileLabel: {
		id: 'servers.modpack-upload.file-label',
		defaultMessage: '.mrpack file',
	},
})

function confirm() {
	if (mrpack.value) emit('confirm', mrpack.value)
}

function show() {
	file.value = null
	processing.value = false
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
