<script setup lang="ts">
import { ButtonStyled, Toggle } from '@modrinth/ui'
import { ref, watch } from 'vue'

import { get as getSettings, set as setSettings } from '@/helpers/settings.ts'
import { useTheming } from '@/store/state'
import { DEFAULT_FEATURE_FLAGS, type FeatureFlag } from '@/store/theme.ts'

const themeStore = useTheming()

const settings = ref(await getSettings())
const options = ref<FeatureFlag[]>(Object.keys(DEFAULT_FEATURE_FLAGS))

function setFeatureFlag(key: string, value: boolean) {
	themeStore.featureFlags[key] = value
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
	<div class="flex flex-col gap-2.5 min-w-[600px]">
		<div v-for="option in options" :key="option" class="flex items-center justify-between">
			<div>
				<h2 class="m-0 text-lg font-semibold text-contrast capitalize">
					{{ option.replaceAll('_', ' ') }}
				</h2>
			</div>
			<div class="flex items-center gap-2">
				<ButtonStyled type="transparent">
					<button
						:disabled="themeStore.getFeatureFlag(option) === DEFAULT_FEATURE_FLAGS[option]"
						@click="setFeatureFlag(option, DEFAULT_FEATURE_FLAGS[option])"
					>
						Reset to default
					</button>
				</ButtonStyled>
				<Toggle
					id="advanced-rendering"
					:model-value="themeStore.getFeatureFlag(option)"
					@update:model-value="() => setFeatureFlag(option, !themeStore.getFeatureFlag(option))"
				/>
			</div>
		</div>
	</div>
</template>
