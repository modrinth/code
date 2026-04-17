<template>
	<section
		v-if="!nativeDecorations && os !== 'MacOS'"
		class="window-controls"
		data-tauri-drag-region-exclude
	>
		<ButtonStyled type="transparent" circular>
			<button class="titlebar-button" @click="() => getCurrentWindow().minimize()">
				<MinimizeIcon />
			</button>
		</ButtonStyled>
		<ButtonStyled type="transparent" circular>
			<button class="titlebar-button" @click="() => getCurrentWindow().toggleMaximize()">
				<RestoreIcon v-if="isMaximized" />
				<MaximizeIcon v-else />
			</button>
		</ButtonStyled>
		<ButtonStyled type="transparent" circular>
			<button class="titlebar-button close" @click="handleClose">
				<XIcon />
			</button>
		</ButtonStyled>
	</section>
</template>

<script setup>
import { MaximizeIcon, MinimizeIcon, RestoreIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled } from '@modrinth/ui'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { saveWindowState, StateFlags } from '@tauri-apps/plugin-window-state'
import { onMounted, onUnmounted, ref } from 'vue'

import { getOS } from '@/helpers/utils.js'

const nativeDecorations = ref(true)
const isMaximized = ref(false)
const os = ref('')

onMounted(async () => {
	os.value = await getOS()

	const settings = await import('@/helpers/settings.ts').then((m) => m.get())
	nativeDecorations.value = settings.native_decorations

	if (os.value !== 'MacOS') {
		await getCurrentWindow().setDecorations(nativeDecorations.value)
	}

	isMaximized.value = await getCurrentWindow().isMaximized()

	const unlisten = await getCurrentWindow().onResized(async () => {
		isMaximized.value = await getCurrentWindow().isMaximized()
	})

	onUnmounted(() => {
		unlisten()
	})
})

const handleClose = async () => {
	await saveWindowState(StateFlags.ALL)
	await getCurrentWindow().close()
}
</script>

<style lang="scss" scoped>
.window-controls {
	position: fixed;
	top: 0;
	right: 0;
	z-index: 10001;
	display: flex;
	flex-direction: row;
	align-items: center;
	height: var(--top-bar-height, 3rem);
	padding-right: 0.5rem;
	gap: 0.25rem;

	.titlebar-button.close:hover {
		background-color: var(--color-red);
		color: var(--color-accent-contrast);
	}
}
</style>
