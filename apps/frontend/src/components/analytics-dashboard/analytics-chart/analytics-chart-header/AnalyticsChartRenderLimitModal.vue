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
				<ButtonStyled type="transparent">
					<button @click="modal?.hide()">
						{{ formatMessage(analyticsChartMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="orange">
					<button class="!shadow-none" @click="confirm">
						{{ formatMessage(analyticsChartMessages.showAll) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { ButtonStyled, NewModal, useVIntl } from '@modrinth/ui'

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
