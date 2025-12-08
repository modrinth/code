<template>
	<div class="flex flex-col gap-6">
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">
				Version type <span class="text-red">*</span>
			</span>
			<Chips
				v-model="draftVersion.release_channel"
				:items="['release', 'alpha', 'beta']"
				:never-empty="false"
				:capitalize="true"
			/>
		</div>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">
				Version number <span class="text-red">*</span>
			</span>
			<input
				id="version-number"
				v-model="draftVersion.version_number"
				placeholder="Enter version number..."
				type="text"
				autocomplete="off"
				maxlength="32"
			/>
			<span> The version number differentiates this specific version from others. </span>
		</div>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast"> Version subtitle </span>
			<input
				id="version-number"
				v-model="draftVersion.version_title"
				placeholder="Enter subtitle..."
				type="text"
				autocomplete="off"
				maxlength="32"
			/>
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast"> Detected loaders </span>
			<input
				id="version-number"
				v-model="draftVersion.version_title"
				placeholder="Enter subtitle..."
				type="text"
				autocomplete="off"
				maxlength="32"
			/>
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast"> Detected versions </span>
			<input
				id="version-number"
				v-model="draftVersion.version_title"
				placeholder="Enter subtitle..."
				type="text"
				autocomplete="off"
				maxlength="32"
			/>
		</div>
	</div>
</template>

<script lang="ts" setup>
import Chips from '@modrinth/ui/src/components/base/Chips.vue'

import { useManageVersion } from '~/composables/versions/manage-version'

const { draftVersion } = useManageVersion()
</script>
