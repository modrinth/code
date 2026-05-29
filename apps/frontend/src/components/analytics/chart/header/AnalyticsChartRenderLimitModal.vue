<template>
	<NewModal
		ref="modal"
		:header="`Show all ${tableProjectCount} lines in graph?`"
		fade="warning"
		width="500px"
		max-width="calc(100vw - 2rem)"
	>
		<p class="m-0 max-w-[32rem] text-primary">
			Showing all selected lines from table may degrade page performance.
		</p>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="transparent">
					<button @click="modal?.hide()">Cancel</button>
				</ButtonStyled>
				<ButtonStyled color="orange">
					<button class="!shadow-none" @click="confirm">Show all</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { ButtonStyled, NewModal } from '@modrinth/ui'

defineProps<{
	tableProjectCount: number
}>()

const emit = defineEmits<{
	confirm: []
}>()

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
