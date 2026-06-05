<script setup lang="ts">
import { SearchIcon } from '@modrinth/assets'
import { ButtonStyled, StyledInput, Toggle } from '@modrinth/ui'
import Fuse from 'fuse.js'
import { computed, ref, shallowReactive } from 'vue'

import {
	DEFAULT_FEATURE_FLAGS,
	type FeatureFlag,
	saveFeatureFlags,
	useFeatureFlags,
} from '~/composables/featureFlags.ts'

const flags = shallowReactive(useFeatureFlags().value)
const searchQuery = ref('')

const allFlags = computed(() => Object.keys(flags) as FeatureFlag[])

function resetFlag(flag: FeatureFlag) {
	flags[flag] = DEFAULT_FEATURE_FLAGS[flag]
	saveFeatureFlags()
}

const fuse = computed(
	() =>
		new Fuse(allFlags.value, {
			threshold: 0.4,
		}),
)

const filteredFlags = computed(() => {
	if (!searchQuery.value.trim()) {
		return allFlags.value
	}
	return fuse.value.search(searchQuery.value).map((result) => result.item)
})

useSeoMeta({
	robots: 'noindex',
})
</script>

<template>
	<div class="mb-2">
		<StyledInput
			v-model="searchQuery"
			type="search"
			:icon="SearchIcon"
			placeholder="Search flags..."
			wrapper-class="w-full rounded-xl bg-bg-raised"
		/>
	</div>
	<div class="mb-6 flex flex-col gap-2">
		<div
			v-for="flag in filteredFlags"
			:key="`flag-${flag}`"
			class="flex flex-row flex-wrap items-center gap-2 rounded-2xl bg-bg-raised p-4"
		>
			<label :for="`toggle-${flag}`" class="flex-1">
				<span class="block font-semibold capitalize">
					{{ flag.replaceAll('_', ' ') }}
				</span>
				<p class="m-0 text-secondary">
					Default:
					<span :class="DEFAULT_FEATURE_FLAGS[flag] === false ? 'text-red' : 'text-green'">
						{{ DEFAULT_FEATURE_FLAGS[flag] }}
					</span>
				</p>
			</label>
			<div class="flex items-center gap-2">
				<ButtonStyled type="transparent">
					<button :disabled="flags[flag] === DEFAULT_FEATURE_FLAGS[flag]" @click="resetFlag(flag)">
						Reset to default
					</button>
				</ButtonStyled>
				<Toggle
					:id="`toggle-${flag}`"
					v-model="flags[flag]"
					@update:model-value="() => saveFeatureFlags()"
				/>
			</div>
		</div>
		<p v-if="filteredFlags.length === 0" class="text-center text-secondary">
			No flags found matching "{{ searchQuery }}"
		</p>
	</div>
</template>
