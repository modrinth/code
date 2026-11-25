<script setup lang="ts">
import { useQuery } from '@tanstack/vue-query'

import { sandboxUnavailableReasonQueryOptions } from '@/features/sandbox/queries'
import SandboxSettingsForm from '@/features/sandbox/sandbox-settings-form/index.vue'

import { useDefaultInstanceSettings } from './use-default-instance-settings'

const { settings } = useDefaultInstanceSettings(
	(value) => ({ ...value.sandbox, extra_paths: [...value.sandbox.extra_paths] }),
	(value) => ({ sandbox: { ...value, extra_paths: [...value.extra_paths] } }),
)
const unavailableReason = useQuery(sandboxUnavailableReasonQueryOptions())
</script>

<template>
	<SandboxSettingsForm
		v-if="settings"
		:settings="settings"
		:unavailable-reason="unavailableReason.data.value"
		@update="(patch) => settings && Object.assign(settings, patch)"
	/>
</template>
