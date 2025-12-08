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

		<template v-if="inferredVersionData?.loaders?.length">
			<div class="flex flex-col">
				<div class="flex items-center justify-between">
					<span class="font-semibold text-contrast">
						{{ selectedIsDetectedLoaders ? 'Detected loaders' : 'Loaders' }}
					</span>

					<ButtonStyled type="transparent" size="standard">
						<button @click="editLoaders">
							<EditIcon />
							Edit
						</button>
					</ButtonStyled>
				</div>

				<div
					class="flex flex-col gap-1.5 gap-y-4 rounded-xl border border-solid border-surface-5 p-3 py-4"
				>
					<div class="flex flex-wrap gap-2">
						<template
							v-for="loader in draftVersion.loaders.map((selectedLoader) =>
								loaders.find((loader) => selectedLoader === loader.name),
							)"
						>
							<TagItem
								v-if="loader"
								:key="`loader-${loader.name}`"
								class="hover:no-underline"
								:style="`--_color: var(--color-platform-${loader.name})`"
							>
								<div v-html="loader.icon"></div>
								{{ formatCategory(loader.name) }}
							</TagItem>
						</template>

						<span v-if="!draftVersion.loaders.length">No loaders selected.</span>
					</div>
				</div>
			</div>
		</template>

		<template v-if="inferredVersionData?.game_versions?.length">
			<div class="flex flex-col">
				<div class="flex items-center justify-between">
					<span class="font-semibold text-contrast">
						{{ selectedIsDetectedVersions ? 'Detected versions' : 'Versions' }}
					</span>

					<ButtonStyled type="transparent" size="standard">
						<button @click="editVersions">
							<EditIcon />
							Edit
						</button>
					</ButtonStyled>
				</div>

				<div
					class="flex flex-col gap-1.5 gap-y-4 rounded-xl border border-solid border-surface-5 p-3 py-4"
				>
					<div class="flex flex-wrap gap-2">
						<TagItem v-for="version in draftVersion.game_versions" :key="version">
							{{ version }}
						</TagItem>

						<span v-if="!draftVersion.game_versions.length">No versions selected.</span>
					</div>
				</div>
			</div>
		</template>
	</div>
</template>

<script lang="ts" setup>
import { EditIcon } from '@modrinth/assets'
import { ButtonStyled, Chips, TagItem } from '@modrinth/ui'
import type MultiStageModal from '@modrinth/ui/src/components/base/MultiStageModal.vue'
import { formatCategory } from '@modrinth/utils'
import { useGeneratedState } from '~/composables/generated'
import { useManageVersion } from '~/composables/versions/manage-version'

const { draftVersion, inferredVersionData } = useManageVersion()

const generatedState = useGeneratedState()
const loaders = computed(() => generatedState.value.loaders)

const createVersionModal = inject<Ref<InstanceType<typeof MultiStageModal>>>('createVersionModal')

const editLoaders = () => {
	createVersionModal?.value?.setStage('edit-loaders')
}
const editVersions = () => {
	createVersionModal?.value?.setStage('edit-mc-versions')
}

const selectedIsDetectedVersions = computed(() => {
	if (!inferredVersionData.value?.game_versions) return false

	const versionsMatch =
		draftVersion.value.game_versions.length === inferredVersionData.value.game_versions.length &&
		draftVersion.value.game_versions.every((version) =>
			inferredVersionData.value?.game_versions?.includes(version),
		)

	return versionsMatch
})

const selectedIsDetectedLoaders = computed(() => {
	if (!inferredVersionData.value?.loaders) return false

	const loadersMatch =
		draftVersion.value.loaders.length === inferredVersionData.value.loaders.length &&
		draftVersion.value.loaders.every((loader) =>
			inferredVersionData.value?.loaders?.includes(loader),
		)

	return loadersMatch
})
</script>
