<template>
	<NewModal ref="modal" header="Moderation shortcuts" :closable="true">
		<div id="moderation-checklist-keybinds-modal">
			<div class="keybinds-sections">
				<div class="grid grid-cols-2 gap-x-12 gap-y-3">
					<div
						v-for="[id, keybind] in Object.entries(keybinds)"
						:key="id"
						class="keybind-item flex flex-wrap items-center justify-between gap-4"
						:class="{
							'col-span-2':
								Object.keys(keybinds).length % 2 === 1 &&
								Object.keys(keybinds)[Object.keys(keybinds).length - 1] === id,
						}"
					>
						<span class="text-sm text-secondary">{{ keybind.description }}</span>
						<ChecklistKeybind
							:definitions="
								(!Array.isArray(keybind.keybind) ? [keybind.keybind] : keybind.keybind).map(
									normalizeKeybind,
								)
							"
							:on-change="
								(definitions) => {
									keybinds[id].keybind = definitions
									saveModerationKeybinds()
								}
							"
						/>
					</div>
				</div>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { normalizeKeybind } from '@modrinth/moderation'
import NewModal from '@modrinth/ui/src/components/modal/NewModal.vue'
import { ref } from 'vue'

import { saveModerationKeybinds } from '#imports'
import ChecklistKeybind from '~/components/ui/moderation/checklist/ChecklistKeybind.vue'

const modal = ref<InstanceType<typeof NewModal>>()
const keybinds = useModerationKeybinds()

function show(event?: MouseEvent) {
	modal.value?.show(event)
}

function hide() {
	modal.value?.hide()
}

defineExpose({
	show,
	hide,
})
</script>

<style scoped lang="scss">
@media (max-width: 768px) {
	.keybinds-sections {
		.grid {
			grid-template-columns: 1fr;
			gap: 0.75rem;
		}
	}
}
</style>
