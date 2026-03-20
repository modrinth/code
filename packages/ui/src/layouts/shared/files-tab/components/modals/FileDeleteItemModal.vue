<template>
	<NewModal ref="modal" fade="danger" header="Delete file" max-width="500px">
		<Admonition type="critical" class="md:min-w-[400px]">
			<template #header>Deleting "{{ item?.name }}"</template>
			This {{ item?.type === 'directory' ? 'folder and all its contents' : 'file' }} will be
			permanently deleted. This action is permament and cannot be undone.
		</Admonition>
		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="hide">
						<XIcon class="h-5 w-5" />
						Cancel
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button @click="handleSubmit">
						<TrashIcon class="h-5 w-5" />
						Delete
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { TrashIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import NewModal from '#ui/components/modal/NewModal.vue'

defineProps<{
	item: {
		name: string
		type: string
	} | null
}>()

const emit = defineEmits<{
	delete: []
}>()

const modal = ref<InstanceType<typeof NewModal>>()

const handleSubmit = () => {
	emit('delete')
	hide()
}

const show = () => {
	modal.value?.show()
}

const hide = () => {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
