<script setup lang="ts">
import { defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { sandboxUnavailableReasonQueryOptions } from '@/features/sandbox/queries'
import SandboxSettingsForm from '@/features/sandbox/sandbox-settings-form/index.vue'
import { edit } from '@/helpers/instance'
import { appSettingsQueryOptions } from '@/helpers/settings'
import type { SandboxOverrides, SandboxSettings } from '@/helpers/types'

import { injectInstanceSettings } from './instance-settings-context'
import SettingsSourceHeader from './settings-source-header.vue'
import { useInstanceSettingsDraft } from './use-instance-settings-draft'
import { useSettingsSource } from './use-settings-source'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const { instance } = injectInstanceSettings()

const messages = defineMessages({
	settingsHeading: {
		id: 'instance.settings.tabs.sandbox.settings-heading',
		defaultMessage: 'Sandboxing settings',
	},
})

const draft = useInstanceSettingsDraft<SandboxOverrides>(
	() => ({
		mode: instance.value.sandbox?.mode ?? null,
		allow_network: instance.value.sandbox?.allow_network ?? null,
		allow_downloads: instance.value.sandbox?.allow_downloads ?? null,
		allow_documents: instance.value.sandbox?.allow_documents ?? null,
		extra_paths: instance.value.sandbox?.extra_paths ?? null,
	}),
	(value) => edit(instance.value.id, { sandbox: value }),
)

const settingsQuery = useQuery(appSettingsQueryOptions())
await settingsQuery.suspense().catch(handleError)

const source = useSettingsSource(draft, () => settingsQuery.data.value!.sandbox)

const unavailableReason = useQuery(sandboxUnavailableReasonQueryOptions())

const effective = computed<SandboxSettings | null>(() => {
	const defaults = settingsQuery.data.value?.sandbox
	if (!defaults) return null
	return {
		mode: draft.value.mode ?? defaults.mode,
		allow_network: draft.value.allow_network ?? defaults.allow_network,
		allow_downloads: draft.value.allow_downloads ?? defaults.allow_downloads,
		allow_documents: draft.value.allow_documents ?? defaults.allow_documents,
		extra_paths: draft.value.extra_paths ?? defaults.extra_paths,
	}
})

function applyPatch(patch: Partial<SandboxSettings>) {
	draft.value = { ...draft.value, ...patch }
}
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
			@update="applyPatch"
		/>
	</div>
</template>
