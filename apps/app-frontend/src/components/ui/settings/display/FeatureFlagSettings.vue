<script setup lang="ts">
import { Button, Toggle } from '@modrinth/ui'
import { ref, watch } from 'vue'

import {
	DEFAULT_FEATURE_FLAGS,
	type FeatureFlag,
	useAppSettings,
} from '@/composables/use-app-settings.ts'
import { get as getSettings, set as setSettings } from '@/helpers/settings.ts'

const appSettings = useAppSettings()

const settings = ref(await getSettings())
const options = ref(Object.keys(DEFAULT_FEATURE_FLAGS) as FeatureFlag[])

function setFeatureFlag(key: FeatureFlag, value: boolean) {
	appSettings.featureFlags[key] = value
	settings.value.feature_flags[key] = value
}

watch(
	settings,
	async () => {
		await setSettings(settings.value)
	},
	{ deep: true },
)
</script>
<template>
	<div class="flex flex-col gap-2.5">
		<div v-for="option in options" :key="option" class="flex items-center justify-between">
			<div>
				<h2 class="m-0 text-lg font-semibold text-contrast capitalize">
					{{ option.replaceAll('_', ' ') }}
				</h2>
			</div>
			<div class="flex items-center gap-2">
				<Button
					type="quiet"
					:disabled="appSettings.getFeatureFlag(option) === DEFAULT_FEATURE_FLAGS[option]"
					@click="setFeatureFlag(option, DEFAULT_FEATURE_FLAGS[option])"
				>
					Reset to default
				</Button>
				<Toggle
					id="advanced-rendering"
					:model-value="appSettings.getFeatureFlag(option)"
					@update:model-value="() => setFeatureFlag(option, !appSettings.getFeatureFlag(option))"
				/>
			</div>
		</div>
	</div>
</template>
