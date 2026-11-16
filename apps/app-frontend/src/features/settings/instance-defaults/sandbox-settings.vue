<script setup lang="ts">
import { useQuery } from '@tanstack/vue-query'

import { getSandboxUnavailableReason } from '@/helpers/settings'

import SandboxSettingsForm from './sandbox-settings-form.vue'
import { useDefaultInstanceSettings } from './use-default-instance-settings'

const { settings } = useDefaultInstanceSettings(
	(value) => ({ ...value.sandbox, extra_paths: [...value.sandbox.extra_paths] }),
	(value) => ({ sandbox: { ...value, extra_paths: [...value.extra_paths] } }),
)
const unavailableReason = useQuery({
	queryKey: ['sandbox-unavailable-reason'],
	queryFn: getSandboxUnavailableReason,
	staleTime: Infinity,
})
</script>

<template>
	<SandboxSettingsForm
		v-if="settings"
		v-model="settings"
		:unavailable-reason="unavailableReason.data.value"
	/>
</template>
