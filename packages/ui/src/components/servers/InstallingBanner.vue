<template>
	<Admonition
		v-if="installation"
		:type="installation.status === 'failed' ? 'critical' : 'info'"
		:dismissible="installation.status === 'failed'"
		:progress="progressValue"
		progress-color="blue"
		:waiting="isWaiting"
		@dismiss="emit('dismiss')"
	>
		<template #header>
			{{ headerLabel }}
		</template>
		{{ installation.status === 'failed' ? errorLabel : descriptionLabel }}
		<template v-if="installation.status === 'failed'" #top-right-actions>
			<ButtonStyled color="red" type="outlined">
				<button
					v-tooltip="retryDisabled ? retryDisabledTooltip : undefined"
					class="!border"
					type="button"
					:disabled="retryDisabled"
					@click="emit('retry')"
				>
					<RotateCounterClockwiseIcon class="size-5" />
					{{ formatMessage(commonMessages.retryButton) }}
				</button>
			</ButtonStyled>
		</template>
	</Admonition>
</template>

<script setup lang="ts">
import { RotateCounterClockwiseIcon } from '@modrinth/assets'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectModrinthServerContext } from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'
import { formatLoaderLabel } from '#ui/utils/loaders'

import Admonition from '../base/Admonition.vue'
import ButtonStyled from '../base/ButtonStyled.vue'

defineProps<{
	retryDisabled?: boolean
	retryDisabledTooltip?: string
}>()

const emit = defineEmits<{
	retry: []
	dismiss: []
}>()

const { formatMessage } = useVIntl()
const { installation } = injectModrinthServerContext()

const messages = defineMessages({
	errorHeader: {
		id: 'servers.installing-banner.error.header',
		defaultMessage: 'Installation failed',
	},
	unknownError: {
		id: 'servers.installing-banner.error.unknown',
		defaultMessage: 'An unexpected error occurred during installation.',
	},
	preparingHeader: {
		id: 'servers.installing-banner.preparing.header',
		defaultMessage: 'Preparing your server',
	},
	installingPlatform: {
		id: 'servers.installing-banner.installing-platform',
		defaultMessage: 'Installing {loader} for Minecraft {version}',
	},
	installingMinecraft: {
		id: 'servers.installing-banner.installing-minecraft',
		defaultMessage: 'Installing Minecraft {version}',
	},
	installingModpack: {
		id: 'servers.installing-banner.installing-modpack',
		defaultMessage: 'Installing modpack',
	},
	installingLocalModpack: {
		id: 'servers.installing-banner.installing-local-modpack',
		defaultMessage: 'Installing {filename}',
	},
	preparingDescription: {
		id: 'servers.installing-banner.description.preparing',
		defaultMessage: 'Preparing your server...',
	},
	applyingDescription: {
		id: 'servers.installing-banner.description.applying',
		defaultMessage: 'Applying your installation changes...',
	},
	durationDescription: {
		id: 'servers.installing-banner.description.duration',
		defaultMessage: 'This installation may take several minutes...',
	},
	controlsDescription: {
		id: 'servers.installing-banner.description.controls',
		defaultMessage: 'Server controls will unlock when installation finishes.',
	},
	stillWorkingDescription: {
		id: 'servers.installing-banner.description.still-working',
		defaultMessage: 'Still working—your installation is in progress...',
	},
})

const headerLabel = computed(() => {
	const current = installation.value
	if (!current) return ''
	if (current.status === 'failed') return formatMessage(messages.errorHeader)

	switch (current.key.type) {
		case 'platform': {
			if (current.key.platform === 'vanilla') {
				return formatMessage(messages.installingMinecraft, {
					version: current.key.game_version,
				})
			}
			return formatMessage(messages.installingPlatform, {
				loader: formatLoaderLabel(current.key.platform),
				version: current.key.game_version,
			})
		}
		case 'modrinth_modpack':
			return formatMessage(messages.installingModpack)
		case 'local_modpack':
			return formatMessage(messages.installingLocalModpack, {
				filename: current.key.filename,
			})
		case 'unknown':
			return formatMessage(messages.preparingHeader)
	}

	return formatMessage(messages.preparingHeader)
})

const descriptionIndex = ref(0)
const installationId = computed(() => installation.value?.id ?? null)

watch(installationId, () => {
	descriptionIndex.value = 0
})

const descriptionLabel = computed(() => {
	switch (descriptionIndex.value) {
		case 0:
			return formatMessage(messages.preparingDescription)
		case 1:
			return formatMessage(messages.applyingDescription)
		case 2:
			return formatMessage(messages.durationDescription)
		default:
			return descriptionIndex.value % 2 === 1
				? formatMessage(messages.controlsDescription)
				: formatMessage(messages.stillWorkingDescription)
	}
})

const errorLabel = computed(() => installation.value?.error ?? formatMessage(messages.unknownError))

const progressValue = computed(() => {
	const current = installation.value
	if (!current || current.status === 'failed') return undefined
	return current.progress == null ? 0 : current.progress / 100
})

const isWaiting = computed(() => {
	const current = installation.value
	if (!current || current.status === 'failed') return false
	return current.progress == null || current.progress <= 0
})

let intervalId: ReturnType<typeof setInterval> | null = null

onMounted(() => {
	intervalId = setInterval(() => {
		if (installation.value?.status === 'pending' || installation.value?.status === 'installing') {
			descriptionIndex.value += 1
		}
	}, 15_000)
})

onUnmounted(() => {
	if (intervalId) clearInterval(intervalId)
})
</script>
