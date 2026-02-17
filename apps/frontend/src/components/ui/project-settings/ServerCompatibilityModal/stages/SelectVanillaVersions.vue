<template>
	<div class="flex w-full flex-col gap-4">
		<label>
			<span class="label__title">Supported MC versions</span>
			<McVersionPicker v-model="supportedGameVersions" no-header :game-versions="gameVersions" />
		</label>
		<div>
			<label>
				<span class="label__title">Recommended MC version</span>
				<Combobox
					v-model="recommendedGameVersion"
					v-tooltip="
						!recommendedOptions.length
							? 'Set supported versions before selecting the recommended version'
							: undefined
					"
					:options="recommendedOptions"
					searchable
					:display-name="(val: string) => val"
					placeholder="Select version"
					:disabled="!recommendedOptions.length"
				/>
				<div class="mt-2 text-secondary">
					Players joining the server from the Modrinth App will connect using this version.
				</div>
			</label>
		</div>
	</div>
</template>

<script setup lang="ts">
import { Combobox } from '@modrinth/ui'
import { computed, watch } from 'vue'

import McVersionPicker from '~/components/ui/create-project-version/components/McVersionPicker.vue'
import { useGeneratedState } from '~/composables/generated'

import { injectServerCompatibilityContext } from '../manage-server-compatibility-modal'

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
