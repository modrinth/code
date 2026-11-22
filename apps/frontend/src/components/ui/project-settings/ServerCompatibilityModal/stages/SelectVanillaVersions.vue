<template>
	<div class="flex w-full flex-col gap-6">
		<DataLossWarningBanner />
		<div class="flex w-full flex-col gap-4">
			<Field label="Supported MC versions">
				<template #default="{ labelId }">
					<McVersionPicker
						v-model="supportedGameVersions"
						no-header
						:game-versions="gameVersions"
						role="group"
						:aria-labelledby="labelId"
					/>
				</template>
			</Field>
			<Field label="Recommended MC version" input-id="compatibility-recommended-version">
				<Combobox
					v-model="recommendedGameVersion"
					v-tooltip="
						!recommendedOptions.length
							? 'Set supported versions before selecting the recommended version'
							: undefined
					"
					:options="recommendedOptions"
					searchable
					:search-input-attrs="{ id: 'compatibility-recommended-version' }"
					:display-name="(val: string) => val"
					placeholder="Select version"
					:disabled="!recommendedOptions.length"
				/>
				<template #bottom>
					<p class="mb-0 mt-2 text-sm text-secondary">
						Players joining the server from the Modrinth App will connect using this version.
					</p>
				</template>
			</Field>
		</div>
	</div>
</template>

<script setup lang="ts">
import { Combobox, Field } from '@modrinth/ui'
import { computed, watch } from 'vue'

import McVersionPicker from '~/components/ui/create-project-version/components/McVersionPicker.vue'
import { useGeneratedState } from '~/composables/generated'
import { injectServerCompatibilityContext } from '~/providers/manage-server-compatibility-modal'

import DataLossWarningBanner from '../DataLossWarningBanner.vue'

const { supportedGameVersions, recommendedGameVersion } = injectServerCompatibilityContext()

const generatedState = useGeneratedState()
const gameVersions = generatedState.value.gameVersions

const recommendedOptions = computed(() =>
	gameVersions
		.filter((v) => v.version_type === 'release')
		.filter((v) => supportedGameVersions.value.includes(v.version))
		.map((v) => ({ label: v.version, value: v.version })),
)

watch(
	() => supportedGameVersions.value,
	(supported) => {
		if (recommendedGameVersion.value && !supported.includes(recommendedGameVersion.value)) {
			recommendedGameVersion.value = null
		}
	},
)
</script>
