<script setup lang="ts">
import { Settings2Icon } from '@modrinth/assets'
import { Button, defineMessage, useVIntl } from '@modrinth/ui'
import { inject } from 'vue'

import {
	appSettingsModalOpenDefaultsKey,
	type AppSettingsDefaultsTab,
} from '@/providers/app-settings-modal'

import { injectInstanceSettings } from './instance-settings-context'

const props = defineProps<{ description: string; tab: AppSettingsDefaultsTab }>()

const { formatMessage } = useVIntl()
const { closeModal } = injectInstanceSettings()
const openDefaultSettings = inject(appSettingsModalOpenDefaultsKey, () => {})
const label = defineMessage({
	id: 'instance.settings.defaults.open-app-settings',
	defaultMessage: 'Manage default settings',
})

function showDefaultSettings() {
	const openSettings = () => openDefaultSettings(props.tab)
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
