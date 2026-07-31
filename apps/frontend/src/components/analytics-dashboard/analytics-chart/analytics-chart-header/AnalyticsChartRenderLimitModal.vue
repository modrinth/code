<template>
	<NewModal
		ref="modal"
		:header="formatMessage(analyticsChartMessages.renderLimitHeader, { count: tableProjectCount })"
		fade="warning"
		width="500px"
		max-width="calc(100vw - 2rem)"
	>
		<p class="m-0 max-w-[32rem] text-primary">
			{{ formatMessage(analyticsChartMessages.renderLimitDescription) }}
		</p>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="quiet" @click="modal?.hide()">
					{{ formatMessage(analyticsChartMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="orange" @click="confirm">
					{{ formatMessage(analyticsChartMessages.showAll) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { Button, NewModal, useVIntl } from '@modrinth/ui'

import { analyticsChartMessages } from '../../analytics-messages'

defineProps<{
	tableProjectCount: number
}>()

const emit = defineEmits<{
	confirm: []
}>()

const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal> | null>(null)

function show(event: MouseEvent) {
	modal.value?.show(event)
}

function hide() {
	modal.value?.hide()
}

function confirm() {
	emit('confirm')
	hide()
}

defineExpose({
	show,
	hide,
})
</script>
