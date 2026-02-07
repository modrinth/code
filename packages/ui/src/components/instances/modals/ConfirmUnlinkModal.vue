<template>
	<NewModal ref="modal" header="Unlink modpack" fade="warning" max-width="500px">
		<div class="flex flex-col gap-6">
			<Admonition type="warning" header="Unlink warning">
				Are you sure you want to unlink the modpack from your instance? Modpack content will remain
				installed, but will no longer be managed.
			</Admonition>
			<span class="text-primary">
				This action is irreversable. You will need to create a new instance with the modpack if you
				change your mind.</span
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
						<UnlinkIcon />
						Unlink
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { UnlinkIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import Admonition from '../../base/Admonition.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import NewModal from '../../modal/NewModal.vue'

const emit = defineEmits<{
	(e: 'unlink'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()

function show() {
	modal.value?.show()
}

function confirm() {
	modal.value?.hide()
	emit('unlink')
}

defineExpose({
	show,
})
</script>
