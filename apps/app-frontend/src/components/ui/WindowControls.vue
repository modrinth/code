<template>
	<section
		v-if="showControls"
		class="flex items-center gap-2 mr-1.5"
		data-tauri-drag-region-exclude
	>
		<IconButton
			:label="formatMessage(messages.minimize)"
			type="quiet"
			class="relative expanded-button"
			@click="() => getCurrentWindow().minimize()"
		>
			<MinimizeIcon aria-hidden="true" />
		</IconButton>
		<IconButton
			:label="formatMessage(messages.toggleMaximize)"
			type="quiet"
			class="relative expanded-button"
			@click="() => getCurrentWindow().toggleMaximize()"
		>
			<RestoreIcon v-if="isMaximized" aria-hidden="true" />
			<MaximizeIcon v-else aria-hidden="true" />
		</IconButton>
		<IconButton
			:label="formatMessage(messages.close)"
			type="quiet"
			color="red"
			class="relative expanded-button close-button"
			@click="handleClose"
		>
			<XIcon aria-hidden="true" />
		</IconButton>
	</section>
</template>

<script setup>
import { MaximizeIcon, MinimizeIcon, RestoreIcon, XIcon } from '@modrinth/assets'
import { defineMessages, IconButton, useVIntl } from '@modrinth/ui'
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
const { formatMessage } = useVIntl()
const messages = defineMessages({
	minimize: { id: 'app.window-controls.minimize', defaultMessage: 'Minimize window' },
	toggleMaximize: {
		id: 'app.window-controls.toggle-maximize',
		defaultMessage: 'Toggle maximize window',
	},
	close: { id: 'app.window-controls.close', defaultMessage: 'Close window' },
})

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
