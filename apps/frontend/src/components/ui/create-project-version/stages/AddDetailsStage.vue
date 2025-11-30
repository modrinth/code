<template>
	<div class="flex w-full max-w-[496px] flex-col gap-6">
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">
				Version type <span class="text-red">*</span>
			</span>
			<Chips
				v-model="versionType"
				:items="['Release', 'Alpha', 'Beta']"
				:never-empty="false"
				:capitalize="true"
			/>
			<!-- :format-label="
					(item) => (item === 'Release' ? formatMessage(messages.release) : item === 'Alpha' ? formatMessage(messages.alpha) : formatMessage(messages.beta))
				" -->
		</div>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">
				Version number <span class="text-red">*</span>
			</span>
			<input
				id="version-number"
				v-model="versionNumber"
				placeholder="Enter version subtitle..."
				type="text"
				autocomplete="off"
				maxlength="32"
			/>
			<span> The version number appears as the version name and used by the API. </span>
		</div>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">
				Version subtitle <span class="text-red">*</span>
			</span>
			<input
				id="version-number"
				v-model="versionSubtitle"
				placeholder="Enter version subtitle..."
				type="text"
				autocomplete="off"
				maxlength="32"
			/>
		</div>
		<div>
			<LoaderPicker v-model="selectedLoaders" :loaders="generatedState.loaders" />
		</div>
	</div>
</template>

<script lang="ts" setup>
import Chips from '@modrinth/ui/src/components/base/Chips.vue'

import LoaderPicker from '../components/LoaderPicker.vue'

const generatedState = useGeneratedState()

type VersionType = 'Release' | 'Alpha' | 'Beta'
const versionType = ref<VersionType>('Release')
const versionNumber = ref('')
const versionSubtitle = ref('')

const selectedLoaders = ref<string[]>([])
</script>
