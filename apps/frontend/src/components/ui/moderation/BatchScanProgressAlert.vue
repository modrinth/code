<template>
	<div
		class="flex flex-col gap-4 rounded-2xl border-[1px] border-solid border-blue bg-highlight-blue p-4"
	>
		<div class="flex flex-row justify-between">
			<div class="flex flex-col text-contrast">
				<span class="text-xl font-semibold">Batch scan in progress</span>
				<span>{{ progress?.complete }} of {{ progress?.total }} projects completed</span>
			</div>
			<ButtonStyled circular color="blue" type="outlined">
				<button class="!px-4" @click="emit('cancel-scan')">Cancel scan</button>
			</ButtonStyled>
		</div>
		<div class="w-full rounded-full bg-highlight-blue">
			<div
				class="h-3 rounded-[inherit] bg-blue"
				:style="`width: ${((progress?.complete ?? 0) / (progress?.total ?? 1)) * 100}%`"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ButtonStyled } from '@modrinth/ui'
import { defineProps } from 'vue'

export interface BatchScanProgress {
	total: number
	complete: number
}

defineProps<{
	progress?: BatchScanProgress
}>()

const emit = defineEmits<{
	(e: 'cancel-scan'): void
}>()
</script>
