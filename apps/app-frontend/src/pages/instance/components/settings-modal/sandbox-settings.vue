<script setup lang="ts">
import { defineMessage, useVIntl } from '@modrinth/ui'

import SandboxSettings from '@/components/ui/settings/instances/SandboxSettings.vue'

import DefaultSettingsIntro from './default-settings-intro.vue'

const { formatMessage } = useVIntl()
const description = defineMessage({
	id: 'instance.settings.tabs.sandbox.default-settings-description',
	defaultMessage: 'Your instance uses the default sandbox permissions unless you override them here.',
})
</script>

<template>
	<div class="flex flex-col gap-6">
		<DefaultSettingsIntro tab="sandbox" :description="formatMessage(description)" />
		<SandboxSettings instance-settings />
	</div>
</template>
