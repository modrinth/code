<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { Button, commonMessages, useVIntl } from '@modrinth/ui'

import SandboxSettingsForm from './sandbox-settings-form.vue'
import { useDefaultInstanceSettings } from './use-default-instance-settings'

const { formatMessage } = useVIntl()
const { settings, settingsQuery } = useDefaultInstanceSettings(
	(value) => ({ sandbox: { ...value.sandbox, folders: [...value.sandbox.folders] } }),
	(value) => ({ sandbox: { ...value.sandbox, folders: [...value.sandbox.folders] } }),
)
</script>

<template>
	<SandboxSettingsForm v-if="settings" v-model="settings.sandbox" />
	<Button v-else-if="settingsQuery.isError.value" @click="settingsQuery.refetch()">
		{{ formatMessage(commonMessages.refreshButton) }}
	</Button>
	<div v-else class="flex items-center gap-2 text-secondary">
		<SpinnerIcon class="size-5 animate-spin" aria-hidden="true" />
		{{ formatMessage(commonMessages.loadingLabel) }}
	</div>
</template>
