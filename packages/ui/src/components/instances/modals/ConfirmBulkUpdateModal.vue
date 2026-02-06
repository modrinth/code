<template>
	<NewModal ref="modal" header="Update projects" fade="warning" max-width="500px">
		<div class="flex flex-col gap-6">
			<Admonition type="warning" header="Update warning">
				Are you sure you want to update {{ count }} project{{ count === 1 ? '' : 's' }} to their
				latest compatible version?
			</Admonition>
			<span class="text-primary">
				Updating can break your instance. New incompatibilities may be introduced. It's recommended
				to update content one-by-one. Proceed with caution and back up your instance first.</span
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
				<ButtonStyled color="orange">
					<button @click="confirm">
						<DownloadIcon />
						Update {{ count }} project{{ count === 1 ? '' : 's' }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { DownloadIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import Admonition from '../../base/Admonition.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import NewModal from '../../modal/NewModal.vue'

defineProps<{
	count: number
}>()

const emit = defineEmits<{
	(e: 'update'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()

function show() {
	modal.value?.show()
}

function confirm() {
	modal.value?.hide()
	emit('update')
}

defineExpose({
	show,
})
</script>
