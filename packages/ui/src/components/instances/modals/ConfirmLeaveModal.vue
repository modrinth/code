<template>
	<NewModal ref="modal" header="Leave page?" fade="warning" max-width="500px">
		<div class="flex flex-col gap-6">
			<Admonition type="critical" header="Upload in progress">
				Files are still being uploaded. Leaving this page will cancel the upload and your changes
				may be lost.
			</Admonition>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="cancel">
						<XIcon />
						Stay on page
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button @click="leave">
						<RightArrowIcon />
						Leave page
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { RightArrowIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import Admonition from '../../base/Admonition.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import NewModal from '../../modal/NewModal.vue'

const modal = ref<InstanceType<typeof NewModal>>()
let resolvePromise: ((value: boolean) => void) | null = null

function prompt(): Promise<boolean> {
	return new Promise((resolve) => {
		resolvePromise = resolve
		modal.value?.show()
	})
}

function leave() {
	modal.value?.hide()
	resolvePromise?.(true)
	resolvePromise = null
}

function cancel() {
	modal.value?.hide()
	resolvePromise?.(false)
	resolvePromise = null
}

defineExpose({ prompt })
</script>
