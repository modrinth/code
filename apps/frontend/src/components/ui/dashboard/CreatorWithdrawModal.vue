<template>
	<NewModal
		ref="withdrawModal"
		:header="currentStageLabel"
		:noblur="true"
		:closable="currentStage !== 'confirmation'"
		:merge-header="!currentStageLabel"
		@onHide="currentStage = undefined"
	>
		<template>
			<p>Test</p>
			<button @click="open('withdraw-limit')">Set withdraw-limit stage</button>
			<button @click="open('payment-method')">Set payment-method stage</button>
			<button @click="open('withdraw-amount')">Set withdraw-amount stage</button>
			<button @click="open('confirmation')">Set confirmation stage</button>
		</template>
	</NewModal>
</template>
<script lang="ts" setup>
import { NewModal } from '@modrinth/ui'

type Stage = 'withdraw-limit' | 'payment-method' | 'withdraw-amount' | 'confirmation'

const stageLabels = readonly<Record<Stage, string | undefined>>({
	'withdraw-limit': 'Withdraw Limit',
	'payment-method': 'Payment Method',
	'withdraw-amount': 'Withdraw Amount',
	confirmation: undefined,
})

const currentStageLabel = computed<string | undefined>(() => {
	if (!currentStage.value) return undefined
	return stageLabels[currentStage.value]
})

const withdrawModal = ref<InstanceType<typeof NewModal> | null>(null)
const currentStage = ref<Stage | undefined>()
function open(stage: Stage) {
	currentStage.value = stage
	withdrawModal.value?.show()
}
</script>
