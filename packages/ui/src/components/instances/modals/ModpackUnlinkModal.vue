<template>
	<NewModal ref="modal" header="Unlink modpack" fade="danger" max-width="500px">
		<div class="flex flex-col gap-6">
			<Admonition type="warning" header="Unlink warning">
				<!-- Unlinking will turn this modpack world into a mod world and merge the modpack's
				mods with your own. -->
				Unlinking will merge all mods, resource packs and or plugins associated with this modpack
				with your own mods.
			</Admonition>

			<span class="text-primary">
				We will automatically create a backup
				<!--of your world-->
				if you continue.
			</span>
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
					<button @click="unlink">
						<UnlinkIcon />
						Unlink modpack
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

function unlink() {
	modal.value?.hide()
	emit('unlink')
}

defineExpose({
	show,
})
</script>
