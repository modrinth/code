<template>
	<section v-if="showControls" class="window-controls" data-tauri-drag-region-exclude>
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
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import { get as getSettings } from '@/helpers/settings.ts'
import { getOS } from '@/helpers/utils.js'

const WINDOW_CONTROLS_WIDTH = '8rem'

const nativeDecorations = ref(true)
const isMaximized = ref(false)
const os = ref('')

const showControls = computed(() => !nativeDecorations.value && os.value !== 'MacOS')
watch(
	showControls,
	(visible) => {
		if (typeof document === 'undefined') return
		if (visible) {
			document.documentElement.style.setProperty('--window-controls-width', WINDOW_CONTROLS_WIDTH)
		} else {
			document.documentElement.style.removeProperty('--window-controls-width')
		}
	},
	{ immediate: true },
)

onMounted(async () => {
	os.value = await getOS()

	const settings = await getSettings()
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
		document.documentElement.style.removeProperty('--window-controls-width')
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
