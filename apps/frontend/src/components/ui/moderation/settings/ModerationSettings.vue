<script setup lang="ts">
import { defineMessages, SettingsPanel, useVIntl } from '@modrinth/ui'

import { useModerationSettings } from '~/composables/moderation'

import { moderationSettingGroups } from './groups'
import ModerationSettingActions from './ModerationSettingActions.vue'

defineProps<{
	embedded?: boolean
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	title: { id: 'moderation.settings.title', defaultMessage: 'General settings' },
})

const settings = useModerationSettings()
</script>

<template>
	<div :class="{ 'universal-card': !embedded }">
		<h2 class="m-0 mb-2.5 text-2xl">{{ formatMessage(messages.title) }}</h2>
		<SettingsPanel :groups="moderationSettingGroups" :store="settings">
			<template #setting-actions="{ setting, store }">
				<ModerationSettingActions :setting="setting" :store="store" />
			</template>
		</SettingsPanel>
	</div>
</template>
