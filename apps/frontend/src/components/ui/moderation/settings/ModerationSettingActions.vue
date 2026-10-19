<script setup lang="ts">
import { moderationSettings } from '@modrinth/moderation'
import {
	Button,
	defineMessages,
	type SettingDefinition,
	type SettingsStore,
	useVIntl,
} from '@modrinth/ui'

/** Extra controls beside a moderation setting, e.g. filling in the local development host. */
defineProps<{
	setting: SettingDefinition
	store: SettingsStore
}>()

const LOCALHOST_HOSTNAME = 'localhost:3000'
const hostname = moderationSettings.General.AlternativeHostname

const { formatMessage } = useVIntl()

const messages = defineMessages({
	localhost: {
		id: 'moderation.settings.localhost',
		defaultMessage: 'Set to {hostname}',
	},
})
</script>

<template>
	<Button
		v-if="setting.id === hostname.id"
		type="outlined"
		size="sm"
		class="shrink-0"
		:disabled="store.get(hostname) === LOCALHOST_HOSTNAME"
		@click="store.set(hostname, LOCALHOST_HOSTNAME)"
	>
		{{ formatMessage(messages.localhost, { hostname: LOCALHOST_HOSTNAME }) }}
	</Button>
</template>
