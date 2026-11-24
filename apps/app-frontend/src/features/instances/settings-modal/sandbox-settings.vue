<script setup lang="ts">
import { Settings2Icon } from '@modrinth/assets'
import { Button, Checkbox, defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import SandboxSettingsForm from '@/features/settings/instance-defaults/sandbox-settings-form/index.vue'
import { injectSettingsActions } from '@/features/settings/settings-actions'
import { edit } from '@/helpers/instance'
import { appSettingsQueryOptions, sandboxUnavailableReasonQueryOptions } from '@/helpers/settings'
import type { SandboxOverrides, SandboxSettings } from '@/helpers/types'

import { injectInstanceSettings } from './instance-settings-context'
import { useInstanceSettingsDraft } from './use-instance-settings-draft'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const { instance, closeModal } = injectInstanceSettings()
const { openInstanceDefaults } = injectSettingsActions()

const messages = defineMessages({
	customTitle: {
		id: 'instance.settings.tabs.sandbox.custom.title',
		defaultMessage: 'Use custom settings',
	},
	customDescription: {
		id: 'instance.settings.tabs.sandbox.custom.description',
		defaultMessage:
			'Use custom sandbox settings for this instance instead of following App settings.',
	},
	defaultSettings: {
		id: 'instance.settings.tabs.sandbox.default-settings',
		defaultMessage: 'Default settings',
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

const unavailableReason = useQuery(sandboxUnavailableReasonQueryOptions())

const custom = computed(() => Object.values(draft.value).some((value) => value !== null))

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

function setCustom(value: boolean) {
	const settings = effective.value
	if (value && settings) {
		draft.value = { ...settings, extra_paths: [...settings.extra_paths] }
	} else {
		draft.value = {
			mode: null,
			allow_network: null,
			allow_downloads: null,
			allow_documents: null,
			extra_paths: null,
		}
	}
}

function applyPatch(patch: Partial<SandboxSettings>) {
	const settings = effective.value
	if (!settings) return
	draft.value = { ...settings, ...patch }
}

function showDefaultSettings() {
	const openSettings = () => openInstanceDefaults('sandbox')
	if (closeModal) {
		closeModal(openSettings)
	} else {
		openSettings()
	}
}
</script>

<template>
	<div class="flex flex-col gap-6">
		<div
			class="flex items-start justify-between gap-4 border-0 border-b border-solid border-surface-5 pb-6"
		>
			<Checkbox
				:model-value="custom"
				:description="formatMessage(messages.customTitle)"
				class="min-w-0 flex-1 !items-start !gap-2.5 [&>span:first-child]:mt-0.5"
				@update:model-value="setCustom"
			>
				<span class="flex min-w-0 flex-col gap-0.5">
					<span class="text-lg font-semibold leading-6 text-contrast">
						{{ formatMessage(messages.customTitle) }}
					</span>
					<span class="leading-6 text-primary">
						{{ formatMessage(messages.customDescription) }}
					</span>
				</span>
			</Checkbox>
			<Button class="shrink-0" @click="showDefaultSettings">
				<Settings2Icon aria-hidden="true" />
				{{ formatMessage(messages.defaultSettings) }}
			</Button>
		</div>
		<SandboxSettingsForm
			v-if="effective"
			:settings="effective"
			instance-settings
			:disabled="!custom"
			:unavailable-reason="unavailableReason.data.value"
			@update="applyPatch"
		/>
	</div>
</template>
