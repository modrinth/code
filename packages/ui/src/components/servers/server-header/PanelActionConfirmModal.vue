<template>
	<NewModal ref="modal" header="Confirming power action" @close="emit('cancel')">
		<div class="flex flex-col gap-4 md:w-[400px]">
			<p class="m-0">
				Are you sure you want to
				<span class="lowercase">{{ pendingAction }}</span>
				the server?
			</p>
			<Checkbox
				v-model="dontAskAgainModel"
				label="Don't ask me again"
				class="text-sm"
				:disabled="!pendingAction"
			/>
			<div class="flex flex-row gap-4">
				<ButtonStyled type="standard" color="brand" @click="emit('confirm')">
					<button :disabled="!pendingAction">
						<CheckIcon class="h-5 w-5" />
						{{ pendingAction }} server
					</button>
				</ButtonStyled>
				<ButtonStyled @click="emit('cancel')">
					<button>
						<XIcon class="h-5 w-5" />
						Cancel
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { CheckIcon, XIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'

import { ButtonStyled, Checkbox, NewModal } from '#ui/components'
import type { PowerAction } from './use-server-power-action'

const props = defineProps<{
	pendingAction: PowerAction | null
	dontAskAgain: boolean
}>()

const emit = defineEmits<{
	confirm: []
	cancel: []
	'update:dontAskAgain': [value: boolean]
}>()

const modal = ref<InstanceType<typeof NewModal> | null>(null)

const dontAskAgainModel = computed({
	get: () => props.dontAskAgain,
	set: (value: boolean) => emit('update:dontAskAgain', value),
})

function show() {
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

defineExpose({
	show,
	hide,
})
</script>
