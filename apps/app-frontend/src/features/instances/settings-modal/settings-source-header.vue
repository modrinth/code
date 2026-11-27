<script setup lang="ts">
import { EditIcon, SettingsIcon, UndoIcon } from '@modrinth/assets'
import { Button, defineMessages, useVIntl } from '@modrinth/ui'

import {
	injectSettingsActions,
	type InstanceDefaultsTab,
} from '@/features/settings/settings-actions'

import { injectInstanceSettings } from './instance-settings-context'
import type { SettingsSource } from './use-settings-source'

const props = defineProps<{ title: string; tab: InstanceDefaultsTab }>()
const source = defineModel<SettingsSource>({ required: true })

const { formatMessage } = useVIntl()
const { closeModal } = injectInstanceSettings()
const { openInstanceDefaults } = injectSettingsActions()

const messages = defineMessages({
	usingDefaults: {
		id: 'instance.settings.source.using-app-defaults',
		defaultMessage: 'Using the defaults from your app settings.',
	},
	usingCustom: {
		id: 'instance.settings.source.using-custom',
		defaultMessage: 'Using custom settings for this instance',
	},
	appSettings: {
		id: 'instance.settings.source.app-settings',
		defaultMessage: 'App settings',
	},
	customize: {
		id: 'instance.settings.source.customize',
		defaultMessage: 'Customize',
	},
	resetToDefaults: {
		id: 'instance.settings.source.reset-to-defaults',
		defaultMessage: 'Reset to app defaults',
	},
})

function showDefaultSettings() {
	const openSettings = () => openInstanceDefaults(props.tab)
	if (closeModal) {
		closeModal(openSettings)
	} else {
		openSettings()
	}
}
</script>

<template>
	<div
		class="flex items-center justify-between gap-4 border-0 border-b border-solid border-surface-5 pb-6"
	>
		<div class="flex min-w-0 flex-col gap-1">
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ title }}
			</h2>
			<p class="m-0 text-secondary">
				{{ formatMessage(source === 'app' ? messages.usingDefaults : messages.usingCustom) }}
			</p>
		</div>
		<div v-if="source === 'app'" class="flex shrink-0 items-center gap-2">
			<Button @click="showDefaultSettings">
				<SettingsIcon aria-hidden="true" />
				{{ formatMessage(messages.appSettings) }}
			</Button>
			<Button type="colored" color="brand" @click="source = 'custom'">
				<EditIcon aria-hidden="true" />
				{{ formatMessage(messages.customize) }}
			</Button>
		</div>
		<Button v-else class="shrink-0" @click="source = 'app'">
			<UndoIcon aria-hidden="true" />
			{{ formatMessage(messages.resetToDefaults) }}
		</Button>
	</div>
</template>
