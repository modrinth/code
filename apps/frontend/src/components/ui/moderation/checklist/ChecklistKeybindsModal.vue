<template>
	<NewModal ref="modal" header="Moderation shortcuts" :closable="true">
		<div>
			<div class="keybinds-sections">
				<div class="grid grid-cols-2 gap-x-12 gap-y-3">
					<div
						v-for="keybind in keybinds"
						:key="keybind.id"
						class="keybind-item flex items-center justify-between gap-4"
						:class="{
							'col-span-2': keybinds.length % 2 === 1 && keybinds[keybinds.length - 1] === keybind,
						}"
					>
						<span class="text-sm text-secondary">{{ keybind.description }}</span>
						<div class="flex items-center gap-1">
							<kbd
								v-for="(key, index) in parseKeybindDisplay(keybind.keybind)"
								:key="`${keybind.id}-key-${index}`"
								class="keybind-key"
							>
								{{ key }}
							</kbd>
						</div>
					</div>
				</div>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { type KeybindListener, keybinds, normalizeKeybind } from '@modrinth/moderation'
import NewModal from '@modrinth/ui/src/components/modal/NewModal.vue'
import { ref } from 'vue'

const modal = ref<InstanceType<typeof NewModal>>()

function parseKeybindDisplay(keybind: KeybindListener['keybind']): string[] {
	const keybinds = Array.isArray(keybind) ? keybind : [keybind]
	const normalized = keybinds[0]
	const def = normalizeKeybind(normalized)

	const keys = []

	if (def.ctrl || def.meta) {
		keys.push(isMac() ? 'CMD' : 'CTRL')
	}
	if (def.shift) keys.push('SHIFT')
	if (def.alt) keys.push('ALT')

	const mainKey = def.key
		.replace('ArrowLeft', '←')
		.replace('ArrowRight', '→')
		.replace('ArrowUp', '↑')
		.replace('ArrowDown', '↓')
		.replace('Enter', '↵')
		.replace('Space', 'SPACE')
		.replace('Escape', 'ESC')
		.toUpperCase()

	keys.push(mainKey)

	return keys
}

function isMac() {
	return navigator.platform.toUpperCase().includes('MAC')
}

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
.keybind-key {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	min-width: 2rem;
	padding: 0.25rem 0.5rem;
	background-color: var(--color-bg);
	border: 1px solid var(--color-divider);
	border-radius: 0.375rem;
	font-size: 0.75rem;
	font-weight: 600;
	text-transform: uppercase;
	color: var(--color-contrast);

	+ .keybind-key {
		margin-left: 0.25rem;
	}
}

.keybind-item {
	min-height: 2rem;
}

@media (max-width: 768px) {
	.keybinds-sections {
		.grid {
			grid-template-columns: 1fr;
			gap: 0.75rem;
		}
	}
}
</style>
