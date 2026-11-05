<template>
	<div class="flex flex-col gap-6">
		<div
			v-if="ctx.setupType.value === 'modpack' && ctx.modpackFile.value"
			class="flex flex-col gap-2"
		>
			<span class="font-semibold text-contrast">{{ formatMessage(messages.mrpackFileLabel) }}</span>
			<ArchiveInput :model-value="ctx.modpackFile.value" disabled />
		</div>
		<template v-else-if="ctx.setupType.value === 'curseforge'">
			<div v-if="ctx.curseforgeModpackFile.value" class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">
					{{ formatMessage(curseforgeMessages.selectedFileLabel) }}
				</span>
				<ArchiveInput :model-value="ctx.curseforgeModpackFile.value" disabled />
			</div>
			<div v-if="ctx.curseforgeServerPackFile.value" class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">
					<IntlFormatted :message-id="curseforgeMessages.serverPackLabel">
						<template #optional="{ children }">
							<span class="text-primary"><component :is="() => children" /></span>
						</template>
					</IntlFormatted>
				</span>
				<ArchiveInput :model-value="ctx.curseforgeServerPackFile.value" disabled />
			</div>
		</template>
		<ModpackUploadProgress v-if="ctx.uploadProgress.value" :progress="ctx.uploadProgress.value" />
	</div>
</template>

<script setup lang="ts">
import IntlFormatted from '#ui/components/base/IntlFormatted.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { injectCreationFlowContext } from '../creation-flow-context'
import { curseforgeMessages } from '../curseforge'
import ArchiveInput from './curseforge-stage/archive-input.vue'
import ModpackUploadProgress from './ModpackUploadProgress.vue'

const ctx = injectCreationFlowContext()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	mrpackFileLabel: {
		id: 'creation-flow.modal.uploading.mrpack-file-label',
		defaultMessage: '.mrpack file',
	},
})
</script>
