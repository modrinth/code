<script setup>
import { onMounted, ref } from 'vue'

import { init_ads_window } from '@/helpers/ads.js'

const adsWrapper = ref(null)

let devicePixelRatioWatcher = null

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

function updateAdPosition() {
	if (adsWrapper.value) {
		init_ads_window()
		initDevicePixelRatioWatcher()
	}
}
</script>

<template>
	<div ref="adsWrapper" class="ad-parent relative flex w-full justify-center cursor-pointer bg-bg">
		<a
			href="https://modrinth.gg/medal?from=app-placeholder"
			target="_blank"
			class="flex max-h-[250px] min-h-[250px] min-w-[300px] max-w-[300px] flex-col gap-4 rounded-[inherit]"
		>
			<img
				src="https://cdn-raw.modrinth.com/medal-modrinth-servers-light-new.webp"
				alt="Host your next server with Modrinth Servers"
				class="hidden light-image rounded-[inherit]"
			/>
			<img
				src="https://cdn-raw.modrinth.com/medal-modrinth-servers-dark-new.webp"
				alt="Host your next server with Modrinth Servers"
				class="dark-image rounded-[inherit]"
			/>
		</a>
	</div>
</template>
<style lang="scss" scoped>
.light,
.light-mode {
	.dark-image {
		display: none;
	}

	.light-image {
		display: block;
	}
}
</style>
