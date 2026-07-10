<template>
	<ButtonStyled color="brand" type="outlined" hover-color-fill="background">
		<button
			v-if="showUpdatePill"
			type="button"
			class="!h-[34px] text-sm !transition-[opacity,transform,background-color,color,filter] !duration-200 ease-out"
			:class="{
				'opacity-0 scale-[0.96]': finishedDownloading && !animateReadyPill,
				'opacity-100 scale-100': finishedDownloading && animateReadyPill,
			}"
			:disabled="isUpdateDownloading"
			:aria-busy="isUpdateDownloading"
			@click="handleUpdateClick"
		>
			<RefreshCwIcon v-if="finishedDownloading" :class="{ 'animate-spin': restarting }" />
			<DownloadIcon v-else />
			<span v-if="isUpdateDownloading">
				{{ formatMessage(messages.downloadingUpdate) }}
				<span class="inline-block w-[3ch] text-right tabular-nums">{{ downloadPercent }}%</span>
			</span>
			<span v-else>{{ updateLabel }}</span>
		</button>
	</ButtonStyled>
</template>

<script setup lang="ts">
import { DownloadIcon, RefreshCwIcon } from '@modrinth/assets'
import { ButtonStyled, defineMessages, useVIntl } from '@modrinth/ui'
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'

import {
	appUpdateState,
	downloadAvailableAppUpdate,
	installAvailableAppUpdate,
} from '@/providers/app-update'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	update: {
		id: 'app.action-bar.update',
		defaultMessage: 'Update',
	},
	downloadingUpdate: {
		id: 'app.action-bar.downloading-update',
		defaultMessage: 'Downloading update',
	},
	reloadToUpdate: {
		id: 'app.action-bar.reload-to-update',
		defaultMessage: 'Reload to update',
	},
})

const {
	downloading,
	downloadPercent,
	downloadProgress,
	finishedDownloading,
	isVisible: isUpdateVisible,
	metered,
	restarting,
} = appUpdateState

const isUpdateDownloading = computed(
	() =>
		downloading.value ||
		(downloadProgress.value > 0 && downloadProgress.value < 1 && !finishedDownloading.value),
)
const showUpdatePill = computed(
	() => isUpdateVisible.value && (finishedDownloading.value || metered.value),
)
const animateReadyPill = ref(false)
const updateLabel = computed(() => {
	if (isUpdateDownloading.value) {
		return formatMessage(messages.downloadingUpdate)
	}

	if (finishedDownloading.value) {
		return formatMessage(messages.reloadToUpdate)
	}

	return formatMessage(messages.update)
})
let readyPillAnimationFrame: number | null = null
watch([showUpdatePill, finishedDownloading], async ([show, ready], [wasShown, wasReady]) => {
	if (readyPillAnimationFrame !== null) {
		cancelAnimationFrame(readyPillAnimationFrame)
		readyPillAnimationFrame = null
	}

	if (!show || !ready) {
		animateReadyPill.value = false
		return
	}

	if (wasShown && wasReady) {
		return
	}

	animateReadyPill.value = false
	await nextTick()
	readyPillAnimationFrame = requestAnimationFrame(() => {
		animateReadyPill.value = true
		readyPillAnimationFrame = null
	})
})
async function handleUpdateClick() {
	if (isUpdateDownloading.value) {
		return
	}

	if (finishedDownloading.value) {
		await installAvailableAppUpdate()
	} else {
		await downloadAvailableAppUpdate()
	}
}

onBeforeUnmount(() => {
	if (readyPillAnimationFrame !== null) {
		cancelAnimationFrame(readyPillAnimationFrame)
	}
})
</script>
