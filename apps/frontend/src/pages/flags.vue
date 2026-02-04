<script setup lang="ts">
import { SearchIcon } from '@modrinth/assets'
import { Toggle } from '@modrinth/ui'
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
	<div class="mx-auto my-4 box-border w-[calc(100%-2rem)] max-w-[800px]">
		<h1 class="mb-4 text-2xl font-bold text-contrast">Feature flags</h1>
		<div class="relative mb-2">
			<SearchIcon
				class="pointer-events-none absolute left-3 top-1/2 size-5 -translate-y-1/2 text-secondary"
			/>
			<input
				v-model="searchQuery"
				type="search"
				placeholder="Search flags..."
				class="w-full rounded-xl bg-bg-raised py-2 pl-10 pr-4"
			/>
		</div>
		<div class="flex flex-col gap-2">
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
				<Toggle
					:id="`toggle-${flag}`"
					v-model="flags[flag]"
					@update:model-value="() => saveFeatureFlags()"
				/>
			</div>
			<p v-if="filteredFlags.length === 0" class="text-center text-secondary">
				No flags found matching "{{ searchQuery }}"
			</p>
		</div>
	</div>
</template>
