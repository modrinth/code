<template>
	<Admonition
		:type="contentError ? 'critical' : 'info'"
		:dismissible="dismissible"
		:progress="progressValue"
		progress-color="blue"
		:waiting="isWaiting"
		@dismiss="emit('dismiss')"
	>
		<template #header>
			{{ headerLabel }}
		</template>
		<template v-if="contentError">
			{{ errorLabel }}
		</template>
		<template v-else-if="effectivePhase">{{ phaseLabel }}</template>
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
				<button class="!border" type="button" @click="emit('retry')">
					<RotateCounterClockwiseIcon class="size-5" />
					{{ formatMessage(commonMessages.retryButton) }}
				</button>
			</ButtonStyled>
		</template>
	</Admonition>
</template>

<script setup lang="ts">
import { RotateCounterClockwiseIcon } from '@modrinth/assets'
import { computed, onMounted, onUnmounted, ref } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

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
	fallbackPhase?: SyncProgress['phase'] | null
	contentError?: ContentError | null
	dismissible?: boolean
}>()

const emit = defineEmits<{
	retry: []
	dismiss: []
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	errorHeader: {
		id: 'servers.installing-banner.error.header',
		defaultMessage: 'Installation failed',
	},
	preparingHeader: {
		id: 'servers.installing-banner.preparing.header',
		defaultMessage: "We're preparing your server",
	},
	invalidLoaderVersionError: {
		id: 'servers.installing-banner.error.invalid-loader-version',
		defaultMessage:
			'The specified loader or Minecraft version could not be installed. It may be invalid or unsupported.',
	},
	unsupportedLoaderVersionError: {
		id: 'servers.installing-banner.error.unsupported-loader-version',
		defaultMessage: 'This version of Minecraft or loader is not yet supported by Modrinth Hosting.',
	},
	internalPlatformError: {
		id: 'servers.installing-banner.error.internal-platform',
		defaultMessage: 'An internal error occurred while installing the platform. Please try again.',
	},
	noPrimaryFileError: {
		id: 'servers.installing-banner.error.no-primary-file',
		defaultMessage:
			'This modpack version does not include a downloadable file. It may have been packaged incorrectly.',
	},
	modpackInstallFailedError: {
		id: 'servers.installing-banner.error.modpack-install-failed',
		defaultMessage: 'The modpack could not be installed. It may be corrupted or incompatible.',
	},
	unknownError: {
		id: 'servers.installing-banner.error.unknown',
		defaultMessage: 'An unexpected error occurred during installation.',
	},
	installingPlatform: {
		id: 'servers.installing-banner.phase.installing-platform',
		defaultMessage: 'Installing platform...',
	},
	installingModpack: {
		id: 'servers.installing-banner.phase.installing-modpack',
		defaultMessage: 'Installing modpack...',
	},
	installingAddons: {
		id: 'servers.installing-banner.phase.installing-addons',
		defaultMessage: 'Installing addons...',
	},
	tickerOrganizingFiles: {
		id: 'servers.installing-banner.ticker.organizing-files',
		defaultMessage: 'Organizing files...',
	},
	tickerDownloadingMods: {
		id: 'servers.installing-banner.ticker.downloading-mods',
		defaultMessage: 'Downloading mods...',
	},
	tickerConfiguringServer: {
		id: 'servers.installing-banner.ticker.configuring-server',
		defaultMessage: 'Configuring server...',
	},
	tickerSettingUpEnvironment: {
		id: 'servers.installing-banner.ticker.setting-up-environment',
		defaultMessage: 'Setting up environment...',
	},
	tickerAddingJava: {
		id: 'servers.installing-banner.ticker.adding-java',
		defaultMessage: 'Adding Java...',
	},
})

const errorLabel = computed(() => {
	const desc = props.contentError?.description?.toLowerCase()
	const step = props.contentError?.step

	if (step === 'modloader') {
		if (desc === 'the specified version may be incorrect') {
			return formatMessage(messages.invalidLoaderVersionError)
		}
		if (desc === 'this version is not yet supported') {
			return formatMessage(messages.unsupportedLoaderVersionError)
		}
		if (desc === 'internal error') {
			return formatMessage(messages.internalPlatformError)
		}
	}

	if (step === 'modpack') {
		if (desc?.includes('no primary file')) {
			return formatMessage(messages.noPrimaryFileError)
		}
		if (desc?.includes('failed to install')) {
			return formatMessage(messages.modpackInstallFailedError)
		}
	}

	return props.contentError?.description ?? formatMessage(messages.unknownError)
})

const effectivePhase = computed(() => props.progress?.phase ?? props.fallbackPhase ?? null)

const headerLabel = computed(() => {
	if (props.contentError) return formatMessage(messages.errorHeader)
	if (effectivePhase.value === 'Addons') return formatMessage(commonMessages.installingContentLabel)
	return formatMessage(messages.preparingHeader)
})

const phaseLabel = computed(() => {
	switch (effectivePhase.value) {
		case 'InstallingLoader':
			return formatMessage(messages.installingPlatform)
		case 'InstallingPack':
			return formatMessage(messages.installingModpack)
		case 'Addons':
			return formatMessage(messages.installingAddons)
		default:
			return formatMessage(commonMessages.installingLabel)
	}
})

const progressValue = computed(() => {
	if (props.contentError) return undefined
	return props.progress ? props.progress.percent / 100 : 0
})

const isWaiting = computed(() => {
	if (props.contentError) return false
	return !props.progress || props.progress.percent <= 0
})

const tickerMessages = computed(() => [
	formatMessage(messages.tickerOrganizingFiles),
	formatMessage(messages.tickerDownloadingMods),
	formatMessage(messages.tickerConfiguringServer),
	formatMessage(messages.tickerSettingUpEnvironment),
	formatMessage(messages.tickerAddingJava),
])

const currentIndex = ref(0)

let intervalId: ReturnType<typeof setInterval> | null = null

onMounted(() => {
	intervalId = setInterval(() => {
		currentIndex.value = (currentIndex.value + 1) % tickerMessages.value.length
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
