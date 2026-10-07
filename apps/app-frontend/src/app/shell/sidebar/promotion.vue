<script setup lang="ts">
import { ArrowBigUpDashIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { onMounted, onScopeDispose, ref } from 'vue'

import { init_ads_window } from '@/helpers/ads.js'

const { formatMessage } = useVIntl()
const messages = defineMessages({
	upgradeToModrinthPlus: {
		id: 'app.nav.upgrade-to-modrinth-plus',
		defaultMessage: 'Upgrade to Modrinth+',
	},
})
const adsWrapper = ref<HTMLElement | null>(null)

let devicePixelRatioWatcher: MediaQueryList | null = null

function initDevicePixelRatioWatcher() {
	if (devicePixelRatioWatcher) {
		devicePixelRatioWatcher.removeEventListener('change', updateAdPosition)
	}

	devicePixelRatioWatcher = window.matchMedia(`(resolution: ${window.devicePixelRatio}dppx)`)
	devicePixelRatioWatcher.addEventListener('change', updateAdPosition)
}

onMounted(() => {
	updateAdPosition()

	window.addEventListener('resize', updateAdPosition)
	initDevicePixelRatioWatcher()
})

onScopeDispose(() => {
	window.removeEventListener('resize', updateAdPosition)
	devicePixelRatioWatcher?.removeEventListener('change', updateAdPosition)
})

function updateAdPosition() {
	if (adsWrapper.value) {
		init_ads_window()
		initDevicePixelRatioWatcher()
	}
}
</script>

<template>
	<a
		href="https://modrinth.plus?app"
		class="absolute bottom-[250px] w-full flex justify-center items-center gap-1 px-4 py-3 text-purple font-medium hover:underline z-10"
		target="_blank"
	>
		<ArrowBigUpDashIcon class="text-2xl" />
		{{ formatMessage(messages.upgradeToModrinthPlus) }}
	</a>
	<div ref="adsWrapper" class="ad-parent relative flex w-full justify-center cursor-pointer bg-surface-1">
		<a
			href="https://modrinth.host/medal?from=app-placeholder"
			target="_blank"
			class="flex max-h-[250px] min-h-[250px] min-w-[300px] max-w-[300px] flex-col gap-4 rounded-[inherit]"
		>
			<img
				src="https://cdn.modrinth.com/modrinth-hosting-medal-light.webp"
				alt="Host your next server with Modrinth Hosting"
				class="hidden rounded-[inherit] [.light_&]:block [.light-mode_&]:block"
			/>
			<img
				src="https://cdn.modrinth.com/modrinth-hosting-medal-dark.webp"
				alt="Host your next server with Modrinth Hosting"
				class="rounded-[inherit] [.light_&]:hidden [.light-mode_&]:hidden"
			/>
		</a>
	</div>
</template>
