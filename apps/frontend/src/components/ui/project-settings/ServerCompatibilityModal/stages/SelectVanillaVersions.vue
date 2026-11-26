<template>
	<div class="flex w-full flex-col gap-6">
		<DataLossWarningBanner />
		<div class="flex w-full flex-col gap-4">
			<Field :label="formatMessage(messages.supportedMcVersions)">
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
			<Field
				:label="formatMessage(messages.recommendedMcVersion)"
				:input-id="recommendedVersionInputId"
			>
				<Combobox
					v-model="recommendedGameVersion"
					v-tooltip="
						!recommendedOptions.length ? formatMessage(messages.selectSupportedVersions) : undefined
					"
					:options="recommendedOptions"
					searchable
					:search-input-id="recommendedVersionInputId"
					:display-name="(val: string) => val"
					:placeholder="formatMessage(messages.selectVersion)"
					:disabled="!recommendedOptions.length"
				/>
				<template #subtext>
					{{ formatMessage(messages.recommendedVersionHelp) }}
				</template>
			</Field>
		</div>
	</div>
</template>

<script setup lang="ts">
import { Combobox, Field, useVIntl } from '@modrinth/ui'
import { computed, useId, watch } from 'vue'

import McVersionPicker from '~/components/ui/create-project-version/components/McVersionPicker.vue'
import { useGeneratedState } from '~/composables/generated'
import { injectServerCompatibilityContext } from '~/providers/manage-server-compatibility-modal'

import DataLossWarningBanner from '../DataLossWarningBanner.vue'
import { compatibilityMessages as messages } from '../messages'

const { formatMessage } = useVIntl()

const { supportedGameVersions, recommendedGameVersion } = injectServerCompatibilityContext()

const recommendedVersionInputId = useId()
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
