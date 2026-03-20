<template>
	<NewModal
		ref="modal"
		:header="`Move ${item?.type === 'directory' ? 'folder' : 'file'}`"
		max-width="500px"
	>
		<form class="space-y-6 md:min-w-[400px]" @submit.prevent="handleSubmit">
			<div class="flex flex-col gap-1">
				<span class="font-semibold text-contrast">Current location</span>
				<span class="text-secondary">{{ `${currentPath}/${item?.name}`.replace('//', '/') }}</span>
			</div>
			<label class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">Destination path</span>
				<StyledInput
					ref="destinationInput"
					v-model="destination"
					placeholder="e.g. /mods"
					wrapper-class="w-full"
				/>
			</label>
		</form>
		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="hide">
						<XIcon class="h-5 w-5" />
						Cancel
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button @click="handleSubmit">
						<RightArrowIcon class="h-5 w-5" />
						Move
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { RightArrowIcon, XIcon } from '@modrinth/assets'
import { nextTick, ref } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import NewModal from '#ui/components/modal/NewModal.vue'

const destinationInput = ref<HTMLInputElement | null>(null)

defineProps<{
	item: { name: string; type: string } | null
	currentPath: string
}>()

const emit = defineEmits<{
	move: [destination: string]
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const destination = ref('')

const handleSubmit = () => {
	const path = destination.value.replace('//', '/')
	const normalized = path.startsWith('/') ? path : `/${path}`
	emit('move', normalized)
	hide()
}

const show = () => {
	destination.value = ''
	modal.value?.show()
	nextTick(() => {
		setTimeout(() => {
			destinationInput.value?.focus()
		}, 100)
	})
}

const hide = () => {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
