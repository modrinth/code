<template>
	<div class="flex flex-col gap-6">
		<McVersionPicker v-model="draftVersion.game_versions" :game-versions="gameVersions" />
		<div v-if="draftVersion.game_versions.length" class="space-y-2">
			<div class="flex items-center justify-between">
				<span class="font-semibold text-contrast"> Added versions </span>
				<ButtonStyled type="transparent" size="standard">
					<button @click="clearAllVersions()">Clear all</button>
				</ButtonStyled>
			</div>
			<div
				class="flex flex-col gap-1.5 gap-y-4 rounded-xl border border-solid border-surface-5 p-3 py-4"
			>
				<div class="flex flex-wrap gap-2">
					<template v-if="draftVersion.game_versions.length">
						<ButtonStyled v-for="version in draftVersion.game_versions" :key="version" type="chip">
							<button class="w-max" @click="toggleVersion(version)">
								{{ version }}
								<XIcon />
							</button>
						</ButtonStyled>
					</template>
					<template v-else>
						<span>No versions selected.</span>
					</template>
				</div>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
import { XIcon } from '@modrinth/assets'
import ButtonStyled from '@modrinth/ui/src/components/base/ButtonStyled.vue'

import { useManageVersion } from '~/composables/versions/manage-version'

import McVersionPicker from '../components/McVersionPicker.vue'

const generatedState = useGeneratedState()
const gameVersions = generatedState.value.gameVersions

const { draftVersion } = useManageVersion()

const toggleVersion = (version: string) => {
	if (draftVersion.value.game_versions.includes(version)) {
		draftVersion.value.game_versions = draftVersion.value.game_versions.filter((v) => v !== version)
	} else {
		draftVersion.value.game_versions.push(version)
	}
}

const clearAllVersions = () => {
	draftVersion.value.game_versions = []
}
</script>
