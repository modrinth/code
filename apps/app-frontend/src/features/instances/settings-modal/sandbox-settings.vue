<script setup lang="ts">
import { defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { sandboxUnavailableReasonQueryOptions } from '@/features/sandbox/queries'
import SandboxSettingsForm from '@/features/sandbox/sandbox-settings-form/index.vue'
import { appSettingsQueryOptions } from '@/helpers/settings'
import type { SandboxOverrides, SandboxSettings } from '@/helpers/types'

import SettingsSourceHeader from './settings-source-header.vue'
import { useSettingsSource } from './use-settings-source'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	settingsHeading: {
		id: 'instance.settings.tabs.sandbox.settings-heading',
		defaultMessage: 'Sandboxing settings',
	},
})

const settingsQuery = useQuery(appSettingsQueryOptions())
await settingsQuery.suspense().catch(handleError)

const { source, values: overrides, update } = useSettingsSource<SandboxOverrides>({
	read: (current) => ({
		mode: current.sandbox?.mode ?? null,
		allow_network: current.sandbox?.allow_network ?? null,
		allow_downloads: current.sandbox?.allow_downloads ?? null,
		allow_documents: current.sandbox?.allow_documents ?? null,
		extra_paths: current.sandbox?.extra_paths ?? null,
	}),
	write: (value) => ({ sandbox: value }),
	defaults: () => settingsQuery.data.value!.sandbox,
})

const unavailableReason = useQuery(sandboxUnavailableReasonQueryOptions())

const effective = computed<SandboxSettings | null>(() => {
	const defaults = settingsQuery.data.value?.sandbox
	if (!defaults) return null
	return {
		mode: overrides.value.mode ?? defaults.mode,
		allow_network: overrides.value.allow_network ?? defaults.allow_network,
		allow_downloads: overrides.value.allow_downloads ?? defaults.allow_downloads,
		allow_documents: overrides.value.allow_documents ?? defaults.allow_documents,
		extra_paths: overrides.value.extra_paths ?? defaults.extra_paths,
	}
})
</script>

<template>
	<div class="flex flex-col gap-6">
		<SettingsSourceHeader
			v-model="source"
			tab="sandbox"
			:title="formatMessage(messages.settingsHeading)"
		/>
		<SandboxSettingsForm
			v-if="effective"
			:settings="effective"
			instance-settings
			:disabled="source === 'app'"
			:unavailable-reason="unavailableReason.data.value"
			@update="update"
		/>
	</div>
</template>
