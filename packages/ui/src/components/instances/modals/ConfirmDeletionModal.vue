<template>
	<NewModal
		ref="modal"
		:header="`Delete ${itemType}${count > 1 ? 's' : ''}`"
		fade="danger"
		max-width="500px"
	>
		<div class="flex flex-col gap-6">
			<Admonition type="warning" header="Deletion warning">
				Removing content from your instance may corrupt worlds where they were used. Are you sure
				you want to continue?
			</Admonition>
			<span class="text-primary">
				This action is irreversable. Consider making a backup of your worlds before
				continuing.</span
			>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled>
					<button @click="modal?.hide()">
						<XIcon />
						Cancel
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button @click="confirm">
						<TrashIcon />
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

import Admonition from '../../base/Admonition.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import NewModal from '../../modal/NewModal.vue'

defineProps<{
	count: number
	itemType: string
}>()

const emit = defineEmits<{
	(e: 'delete'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()

function show() {
	modal.value?.show()
}

function confirm() {
	modal.value?.hide()
	emit('delete')
}

defineExpose({
	show,
})
</script>
