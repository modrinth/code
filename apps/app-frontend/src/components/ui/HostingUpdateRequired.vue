<script setup lang="ts">
import { DownloadIcon, ExcitedRinthbot, RefreshCwIcon, ServerStackIcon } from '@modrinth/assets'
import { Button, commonMessages, defineMessages, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import {
	appUpdateState,
	downloadAvailableAppUpdate,
	installAvailableAppUpdate,
} from '@/providers/app-update'
import { useRootBreadcrumb } from '@/providers/breadcrumbs'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'app.hosting.update-required.title',
		defaultMessage: 'Modrinth App update required',
	},
	description: {
		id: 'app.hosting.update-required.description',
		defaultMessage: 'You need to update to use Modrinth Hosting through the Modrinth App',
	},
	downloadToUpdate: {
		id: 'app.hosting.update-required.download',
		defaultMessage: 'Download to update',
	},
	downloadingUpdate: {
		id: 'app.action-bar.downloading-update',
		defaultMessage: 'Downloading update',
	},
	reloadToUpdate: {
		id: 'app.action-bar.reload-to-update',
		defaultMessage: 'Reload to update',
	},
	excitedRinthbotAlt: {
		id: 'app.hosting.update-required.rinthbot-alt',
		defaultMessage: 'Excited Modrinth Bot',
	},
})

useRootBreadcrumb({
	slot: 'root',
	id: 'servers',
	label: () => formatMessage(commonMessages.serversLabel),
	to: '/hosting/manage/',
	visual: { type: 'icon', component: ServerStackIcon },
})

const { downloading, downloadPercent, downloadProgress, finishedDownloading } = appUpdateState

const isUpdateDownloading = computed(
	() =>
		downloading.value ||
		(downloadProgress.value > 0 && downloadProgress.value < 1 && !finishedDownloading.value),
)

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
</script>

<template>
	<div class="box-border flex min-h-full items-center justify-center p-4">
		<div class="relative mx-auto w-full max-w-xl pt-28">
			<img
				:src="ExcitedRinthbot"
				:alt="formatMessage(messages.excitedRinthbotAlt)"
				class="absolute right-8 top-0 h-28 w-auto md:right-20"
			/>
			<div class="relative flex flex-col gap-5 rounded-lg bg-bg-raised p-7 shadow-lg">
				<div
					class="absolute left-0 top-0 h-px w-full bg-gradient-to-r from-transparent via-green-500 to-transparent opacity-40"
					style="
						background: linear-gradient(
							to right,
							transparent 2rem,
							var(--color-green) calc(100% - 13rem),
							var(--color-green) calc(100% - 5rem),
							transparent calc(100% - 2rem)
						);
					"
				></div>

				<div class="flex flex-col gap-5">
					<h1 class="m-0 text-3xl font-extrabold">
						{{ formatMessage(messages.title) }}
					</h1>
					<p class="m-0 text-lg">
						{{ formatMessage(messages.description) }}
					</p>
					<Button
						type="colored"
						color="brand"
						:disabled="isUpdateDownloading"
						:aria-busy="isUpdateDownloading"
						@click="handleUpdateClick"
					>
						<RefreshCwIcon v-if="finishedDownloading" />
						<DownloadIcon v-else />
						<span v-if="isUpdateDownloading">
							{{ formatMessage(messages.downloadingUpdate) }}
							<span class="inline-block w-[3ch] text-right tabular-nums">
								{{ downloadPercent }}%
							</span>
						</span>
						<span v-else-if="finishedDownloading">
							{{ formatMessage(messages.reloadToUpdate) }}
						</span>
						<span v-else>{{ formatMessage(messages.downloadToUpdate) }}</span>
					</Button>
				</div>
			</div>
		</div>
	</div>
</template>
