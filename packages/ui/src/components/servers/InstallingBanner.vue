<template>
	<Admonition type="info" show-actions-underneath>
		<template #icon>
			<slot name="icon">
				<SpinnerIcon class="h-6 w-6 flex-none animate-spin text-brand-blue" />
			</slot>
		</template>
		<template #header>We're preparing your server!</template>
		<template v-if="progress">{{ phaseLabel }}</template>
		<div v-else class="ticker-container">
			<div class="ticker-content">
				<div
					v-for="(message, index) in tickerMessages"
					:key="message"
					class="ticker-item"
					:class="{ active: index === currentIndex % tickerMessages.length }"
				>
					{{ message }}
				</div>
			</div>
		</div>
		<template #actions>
			<ProgressBar
				v-if="progress"
				:progress="progress.percent"
				:max="100"
				color="blue"
				full-width
			/>
			<ProgressBar v-else :progress="0" :max="1" color="blue" full-width waiting />
		</template>
	</Admonition>
</template>

<script setup lang="ts">
import SpinnerIcon from '@modrinth/assets/icons/spinner.svg'
import { computed, onMounted, onUnmounted, ref } from 'vue'

import Admonition from '../base/Admonition.vue'
import ProgressBar from '../base/ProgressBar.vue'

export interface SyncProgress {
	phase: 'Analyzing' | 'InstallingPack' | 'InstallingLoader' | 'Addons'
	percent: number
}

const props = defineProps<{
	progress?: SyncProgress | null
}>()

const phaseLabel = computed(() => {
	switch (props.progress?.phase) {
		case 'InstallingLoader':
			return 'Installing platform...'
		case 'InstallingPack':
			return 'Installing modpack...'
		case 'Addons':
			return 'Installing addons...'
		default:
			return 'Installing...'
	}
})

const tickerMessages = [
	'Organizing files...',
	'Downloading mods...',
	'Configuring server...',
	'Setting up environment...',
	'Adding Java...',
]

const currentIndex = ref(0)

let intervalId: ReturnType<typeof setInterval> | null = null

onMounted(() => {
	intervalId = setInterval(() => {
		currentIndex.value = (currentIndex.value + 1) % tickerMessages.length
	}, 3000)
})

onUnmounted(() => {
	if (intervalId) {
		clearInterval(intervalId)
	}
})
</script>

<style scoped>
.ticker-container {
	height: 20px;
	width: 100%;
	position: relative;
}

.ticker-content {
	position: relative;
	width: 100%;
}

.ticker-item {
	position: absolute;
	top: 0;
	left: 0;
	width: 100%;
	height: 20px;
	display: flex;
	align-items: center;
	white-space: nowrap;
	color: var(--color-secondary-text);
	opacity: 0;
	transform: scale(0.9);
	filter: blur(4px);
	transition: all 0.3s ease-in-out;
}

.ticker-item.active {
	opacity: 1;
	transform: scale(1);
	filter: blur(0);
}
</style>
