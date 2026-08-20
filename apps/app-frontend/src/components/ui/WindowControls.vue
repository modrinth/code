<template>
	<section
		v-if="showControls"
		class="flex items-center gap-2 mr-1.5"
		data-tauri-drag-region-exclude
	>
		<IconButton
			type="quiet"
			label="Minimize window"
			class="relative expanded-button"
			@click="() => getCurrentWindow().minimize()"
		>
			<MinimizeIcon />
		</IconButton>
		<IconButton
			type="quiet"
			label="Toggle maximize window"
			class="relative expanded-button"
			@click="() => getCurrentWindow().toggleMaximize()"
		>
			<RestoreIcon v-if="isMaximized" />
			<MaximizeIcon v-else />
		</IconButton>
		<IconButton
			type="quiet"
			label="Close window"
			class="relative expanded-button close-button"
			@click="handleClose"
		>
			<XIcon />
		</IconButton>
	</section>
</template>

<script setup>
import { MaximizeIcon, MinimizeIcon, RestoreIcon, XIcon } from '@modrinth/assets'
import { IconButton } from '@modrinth/ui'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { saveWindowState, StateFlags } from '@tauri-apps/plugin-window-state'
import { computed, onMounted, onUnmounted, ref } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import { get as getSettings } from '@/helpers/settings.ts'
import { getOS } from '@/helpers/utils.js'

const appSettings = useAppSettings()

const nativeDecorations = ref(true)
const isMaximized = ref(false)
const os = ref('')

const alwaysShowAppControls = computed(() => appSettings.getFeatureFlag('always_show_app_controls'))

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
