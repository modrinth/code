<script setup lang="ts">
import { Settings2Icon } from '@modrinth/assets'
import { Button, defineMessage, useVIntl } from '@modrinth/ui'

import {
	injectSettingsActions,
	type InstanceDefaultsTab,
} from '@/features/settings/settings-actions'

import { injectInstanceSettings } from './instance-settings-context'

const props = defineProps<{ description: string; tab: InstanceDefaultsTab }>()

const { formatMessage } = useVIntl()
const { closeModal } = injectInstanceSettings()
const { openInstanceDefaults } = injectSettingsActions()
const label = defineMessage({
	id: 'instance.settings.defaults.open-app-settings',
	defaultMessage: 'Manage default settings',
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
	<div class="flex items-center justify-between gap-4">
		<p class="m-0 min-w-0 text-secondary">{{ description }}</p>
		<Button class="shrink-0" @click="showDefaultSettings">
			<Settings2Icon />
			{{ formatMessage(label) }}
		</Button>
	</div>
</template>
