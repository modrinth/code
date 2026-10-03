<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.editSection, { section })"
		:width="width"
		:disable-close="saving"
		:on-hide="() => emit('cancel')"
		no-padding
		actions-divider
		scrollable
	>
		<div class="p-6">
			<slot />
		</div>
		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="quiet" :disabled="saving" @click="modal?.hide()">
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="brand"
					:loading="saving"
					:disabled="!canSave"
					@click="emit('save')"
				>
					{{ formatMessage(messages.saveProject) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { Button, commonMessages, NewModal, useVIntl } from '@modrinth/ui'
import { useTemplateRef } from 'vue'

import { projectReviewMessages as messages } from '../../messages'

withDefaults(
	defineProps<{
		section: string
		saving: boolean
		canSave: boolean
		width?: string
	}>(),
	{ width: '48rem' },
)

const emit = defineEmits<{
	cancel: []
	save: []
}>()

const { formatMessage } = useVIntl()
const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')

defineExpose({
	show: () => modal.value?.show(),
	hide: () => modal.value?.hide(),
})
</script>
