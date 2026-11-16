<script setup lang="ts">
import { defineMessage, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import SandboxSettingsForm from '@/features/settings/instance-defaults/sandbox-settings-form.vue'
import { edit } from '@/helpers/instance'
import { appSettingsQueryOptions, getSandboxUnavailableReason } from '@/helpers/settings'
import type { SandboxOverrides, SandboxSettings } from '@/helpers/types'

import DefaultSettingsIntro from './default-settings-intro.vue'
import { injectInstanceSettings } from './instance-settings-context'
import ResetToDefaultButton from './reset-to-default-button.vue'
import { useInstanceSettingsDraft } from './use-instance-settings-draft'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const { instance } = injectInstanceSettings()

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

const unavailableReason = useQuery({
	queryKey: ['sandbox-unavailable-reason'],
	queryFn: getSandboxUnavailableReason,
	staleTime: Infinity,
})

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

function sameValue(a: unknown, b: unknown) {
	return JSON.stringify(a) === JSON.stringify(b)
}

function setField<K extends keyof SandboxSettings>(key: K, value: SandboxSettings[K]) {
	const defaultValue = settingsQuery.data.value?.sandbox[key]
	const saved = (instance.value.sandbox?.[key] ?? null) as SandboxOverrides[K]
	if (sameValue(value, saved ?? defaultValue)) {
		draft.value[key] = saved
	} else if (sameValue(value, defaultValue)) {
		draft.value[key] = null
	} else {
		draft.value[key] = value
	}
}

function applyPatch(patch: Partial<SandboxSettings>) {
	for (const [key, value] of Object.entries(patch) as [keyof SandboxSettings, never][]) {
		setField(key, value)
	}
}

const description = defineMessage({
	id: 'instance.settings.tabs.sandbox.default-settings-description',
	defaultMessage:
		'Your instance uses the default sandbox permissions unless you override them here.',
})
</script>

<template>
	<div class="flex flex-col gap-6">
		<DefaultSettingsIntro tab="sandbox" :description="formatMessage(description)" />
		<SandboxSettingsForm
			v-if="effective"
			:settings="effective"
			instance-settings
			:unavailable-reason="unavailableReason.data.value"
			@update="applyPatch"
		>
			<template #reset="{ field, label }">
				<ResetToDefaultButton
					:disabled="draft[field] === null"
					:setting="label"
					@reset="draft[field] = null"
				/>
			</template>
		</SandboxSettingsForm>
	</div>
</template>
