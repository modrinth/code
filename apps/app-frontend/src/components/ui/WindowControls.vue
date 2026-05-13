<template>
	<section
		v-if="showControls"
		class="flex items-center gap-2 mr-1.5"
		data-tauri-drag-region-exclude
	>
		<ButtonStyled type="transparent" circular>
			<button class="relative expanded-button" @click="() => getCurrentWindow().minimize()">
				<MinimizeIcon />
			</button>
		</ButtonStyled>
		<ButtonStyled type="transparent" circular>
			<button class="relative expanded-button" @click="() => getCurrentWindow().toggleMaximize()">
				<RestoreIcon v-if="isMaximized" />
				<MaximizeIcon v-else />
			</button>
		</ButtonStyled>
		<ButtonStyled
			type="transparent"
			color="red"
			color-fill="none"
			hover-color-fill="background"
			circular
		>
			<button class="relative expanded-button close-button" @click="handleClose">
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
import { computed, onMounted, onUnmounted, ref } from 'vue'

import { get as getSettings } from '@/helpers/settings.ts'
import { getOS } from '@/helpers/utils.js'
import { useTheming } from '@/store/state'

const themeStore = useTheming()

const nativeDecorations = ref(true)
const isMaximized = ref(false)
const os = ref('')

const alwaysShowAppControls = computed(() => themeStore.getFeatureFlag('always_show_app_controls'))

const showControls = computed(
	() =>
		alwaysShowAppControls.value ||
		(!nativeDecorations.value && (os.value === 'Windows' || os.value === 'Linux')),
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
	})
})

const handleClose = async () => {
	await saveWindowState(StateFlags.ALL)
	await getCurrentWindow().close()
}
</script>
<style scoped>
.expanded-button::before {
	inset: -9px -6px;
	content: '';
	position: absolute;
}

.expanded-button.close-button::before {
	inset: -9px -9px -9px -6px;
}
</style>
