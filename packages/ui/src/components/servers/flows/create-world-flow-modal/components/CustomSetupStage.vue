<template>
	<div class="space-y-6">
		<div v-if="!hideLoaderFields" class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">Content loader</span>
			<Chips
				v-model="selectedLoader"
				:items="loaderItems"
				:format-label="capitalize"
				:never-empty="false"
			/>
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">Game version</span>
			<!-- TODO: Game version options should come from tags prop -->
			<Combobox
				v-model="selectedGameVersion"
				:options="gameVersionOptions"
				searchable
				placeholder="Select game version"
			/>
			<span class="text-sm text-secondary">It is recommended to use the latest version.</span>
		</div>

		<div v-if="!hideLoaderFields" class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">Loader version</span>
			<Chips
				v-model="loaderVersionType"
				:items="loaderVersionTypeItems"
				:format-label="capitalize"
			/>
			<div v-if="loaderVersionType === 'other'">
				<!-- TODO: Loader version options should come from tags prop -->
				<Combobox
					v-model="selectedLoaderVersion"
					:options="loaderVersionOptions"
					searchable
					placeholder="Select loader version"
				/>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import Chips from '../../../../base/Chips.vue'
import Combobox, { type ComboboxOption } from '../../../../base/Combobox.vue'
import { injectCreateWorldContext } from '../create-world-context'
import type { LoaderVersionType } from '../create-world-context'

const {
	selectedLoader,
	selectedGameVersion,
	loaderVersionType,
	selectedLoaderVersion,
	hideLoaderFields,
} = injectCreateWorldContext()

const loaderItems = ['fabric', 'neoforge', 'forge', 'quilt']
const loaderVersionTypeItems: LoaderVersionType[] = ['stable', 'latest', 'other']

const capitalize = (item: string) => item.charAt(0).toUpperCase() + item.slice(1)

// TODO: Game version options should come from tags prop
const gameVersionOptions = computed<ComboboxOption<string>[]>(() => [
	{ value: '1.21.4', label: '1.21.4' },
	{ value: '1.21.3', label: '1.21.3' },
	{ value: '1.21.2', label: '1.21.2' },
	{ value: '1.21.1', label: '1.21.1' },
	{ value: '1.21', label: '1.21' },
	{ value: '1.20.6', label: '1.20.6' },
	{ value: '1.20.4', label: '1.20.4' },
	{ value: '1.20.2', label: '1.20.2' },
	{ value: '1.20.1', label: '1.20.1' },
	{ value: '1.20', label: '1.20' },
])

// TODO: Loader version options should come from tags prop
const loaderVersionOptions = computed<ComboboxOption<string>[]>(() => [
	{ value: '0.16.9', label: '0.16.9' },
	{ value: '0.16.8', label: '0.16.8' },
	{ value: '0.16.7', label: '0.16.7' },
])
</script>
