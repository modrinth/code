<script setup lang="ts">
import { defineMessages, injectNotificationManager, Toggle, useVIntl } from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import { edit, get_global_synced_options } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

import { instanceKeys } from '../../query-options'
import { injectInstanceSettings } from './instance-settings-context'

const { instance } = injectInstanceSettings()
const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const queryClient = useQueryClient()

const messages = defineMessages({
	intro: {
		id: 'instance.settings.tabs.behavior.description',
		defaultMessage: 'Choose which tabs appear on this instance.',
	},
	files: {
		id: 'instance.settings.tabs.behavior.files',
		defaultMessage: 'Show Files tab',
	},
	worlds: {
		id: 'instance.settings.tabs.behavior.worlds',
		defaultMessage: 'Show Worlds tab',
	},
	screenshots: {
		id: 'instance.settings.tabs.behavior.screenshots',
		defaultMessage: 'Show Screenshots tab',
	},
	screenshotsRequired: {
		id: 'instance.settings.tabs.behavior.screenshots.required',
		defaultMessage:
			'The Screenshots tab cannot be hidden while the global Screenshots page is turned off.',
	},
})

type InstanceTab = keyof GameInstance['visible_tabs']

const rows: Array<{
	tab: InstanceTab
	title: keyof typeof messages
}> = [
	{ tab: 'files', title: 'files' },
	{ tab: 'worlds', title: 'worlds' },
	{ tab: 'screenshots', title: 'screenshots' },
]

const globalSyncedOptionsQuery = useQuery({
	queryKey: ['global-synced-options'],
	queryFn: get_global_synced_options,
})
const globalScreenshotsEnabled = computed(() => globalSyncedOptionsQuery.data.value?.screenshots)
const saving = ref(false)

function isTabVisible(tab: InstanceTab) {
	if (tab === 'screenshots' && globalScreenshotsEnabled.value === false) return true
	return instance.value.visible_tabs[tab]
}

function disabledReason(tab: InstanceTab) {
	if (tab === 'screenshots' && globalScreenshotsEnabled.value === false) {
		return formatMessage(messages.screenshotsRequired)
	}
	return undefined
}

async function setTabVisible(tab: InstanceTab, visible: boolean) {
	if (disabledReason(tab) || saving.value) return

	const instanceId = instance.value.id
	const detailKey = instanceKeys.detail(instanceId)
	const listKey = instanceKeys.list()
	const previousTabs = instance.value.visible_tabs
	const visibleTabs = { ...previousTabs, [tab]: visible }
	const applyTabs = (current: GameInstance): GameInstance => ({
		...current,
		visible_tabs: visibleTabs,
	})

	saving.value = true
	await Promise.all([
		queryClient.cancelQueries({ queryKey: detailKey }),
		queryClient.cancelQueries({ queryKey: listKey }),
	])
	queryClient.setQueryData<GameInstance>(detailKey, (current) =>
		applyTabs(current ?? instance.value),
	)
	queryClient.setQueryData<GameInstance[]>(listKey, (instances) =>
		instances?.map((candidate) => (candidate.id === instanceId ? applyTabs(candidate) : candidate)),
	)

	try {
		await edit(instanceId, { visible_tabs: visibleTabs })
	} catch (error) {
		const rollbackTabs = (current: GameInstance): GameInstance => ({
			...current,
			visible_tabs: previousTabs,
		})
		queryClient.setQueryData<GameInstance>(detailKey, (current) =>
			current ? rollbackTabs(current) : current,
		)
		queryClient.setQueryData<GameInstance[]>(listKey, (instances) =>
			instances?.map((candidate) =>
				candidate.id === instanceId ? rollbackTabs(candidate) : candidate,
			),
		)
		handleError(error)
	} finally {
		saving.value = false
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: detailKey }),
			queryClient.invalidateQueries({ queryKey: listKey }),
		])
	}
}
</script>

<template>
	<div class="flex flex-col gap-6">
		<p class="m-0 text-secondary">
			{{ formatMessage(messages.intro) }}
		</p>

		<div class="flex flex-col gap-4">
			<div v-for="row in rows" :key="row.tab" class="flex items-center justify-between gap-6">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages[row.title]) }}
				</h2>
				<span v-tooltip="disabledReason(row.tab)" class="flex shrink-0">
					<Toggle
						:id="`show-${row.tab}-tab`"
						:model-value="isTabVisible(row.tab)"
						:disabled="saving || !!disabledReason(row.tab)"
						@update:model-value="(visible) => setTabVisible(row.tab, visible)"
					/>
				</span>
			</div>
		</div>
	</div>
</template>
