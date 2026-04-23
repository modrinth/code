<template>
	<Admonition
		:type="contentError ? 'critical' : 'info'"
		:progress="!contentError ? (progress ? progress.percent / 100 : 0) : undefined"
		progress-color="blue"
		:waiting="!contentError && !progress"
	>
		<template #icon>
			<slot v-if="!contentError" name="icon">
				<SpinnerIcon class="h-6 w-6 flex-none animate-spin text-brand-blue" />
			</slot>
		</template>
		<template #header>
			{{ contentError ? 'Installation failed' : "We're preparing your server" }}
		</template>
		<template v-if="contentError">
			{{ errorLabel }}
		</template>
		<template v-else-if="progress">{{ phaseLabel }}</template>
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
		<template v-if="contentError" #top-right-actions>
			<ButtonStyled color="red" type="outlined">
				<button class="!border" @click="emit('retry')">
					<RotateCounterClockwiseIcon class="size-5" />
					Retry
				</button>
			</ButtonStyled>
		</template>
	</Admonition>
</template>

<script setup lang="ts">
import { RotateCounterClockwiseIcon } from '@modrinth/assets'
import SpinnerIcon from '@modrinth/assets/icons/spinner.svg'
import { computed, onMounted, onUnmounted, ref } from 'vue'

import Admonition from '../base/Admonition.vue'
import ButtonStyled from '../base/ButtonStyled.vue'

export interface SyncProgress {
	phase: 'Analyzing' | 'InstallingPack' | 'InstallingLoader' | 'Addons'
	percent: number
}

export interface ContentError {
	step: string
	description: string
}

const props = defineProps<{
	progress?: SyncProgress | null
	contentError?: ContentError | null
}>()

const emit = defineEmits<{
	retry: []
}>()

const errorLabel = computed(() => {
	const desc = props.contentError?.description?.toLowerCase()
	const step = props.contentError?.step

	if (step === 'modloader') {
		if (desc === 'the specified version may be incorrect') {
			return 'The specified loader or Minecraft version could not be installed. It may be invalid or unsupported.'
		}
		if (desc === 'this version is not yet supported') {
			return 'This version of Minecraft or loader is not yet supported by Modrinth Hosting.'
		}
		if (desc === 'internal error') {
			return 'An internal error occurred while installing the platform. Please try again.'
		}
	}

	if (step === 'modpack') {
		if (desc?.includes('no primary file')) {
			return 'This modpack version does not include a downloadable file. It may have been packaged incorrectly.'
		}
		if (desc?.includes('failed to install')) {
			return 'Failed to install the modpack. It may be corrupted or incompatible.'
		}
	}

	return props.contentError?.description ?? 'An unexpected error occurred during installation.'
})

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
