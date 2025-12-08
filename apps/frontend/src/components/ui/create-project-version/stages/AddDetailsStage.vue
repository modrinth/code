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
				<div class="flex justify-between">
					<span class="font-semibold text-contrast"> Detected loaders </span>

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
							<div
								v-if="loader"
								:key="`loader-${loader.name}`"
								:style="`--_icon: var(--color-platform-${loader.name}); color: var(--color-platform-${loader.name})`"
								class="flex items-center gap-2 rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1 text-sm font-medium"
							>
								<div v-html="loader.icon" class="grid place-content-center"></div>
								{{ formatCategory(loader.name) }}
							</div>
						</template>
					</div>
				</div>
			</div>
		</template>

		<template v-if="inferredVersionData?.game_versions?.length">
			<div class="flex flex-col">
				<div class="flex justify-between">
					<span class="font-semibold text-contrast"> Detected versions </span>

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
						<div
							v-for="version in draftVersion.game_versions"
							:key="version"
							class="flex items-center gap-2 rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1 text-sm font-medium"
						>
							{{ version }}
						</div>
					</div>
				</div>
			</div>
		</template>
	</div>
</template>

<script lang="ts" setup>
import { EditIcon } from '@modrinth/assets'
import { ButtonStyled, Chips } from '@modrinth/ui'
import type MultiStageModal from '@modrinth/ui/src/components/base/MultiStageModal.vue'
import { formatCategory } from '@modrinth/utils'
import { useGeneratedState } from '~/composables/generated'
import { useManageVersion } from '~/composables/versions/manage-version'

const { draftVersion, inferredVersionData } = useManageVersion()

const generatedState = useGeneratedState()
const loaders = computed(() => generatedState.value.loaders)

const createVersionModal = inject<Ref<InstanceType<typeof MultiStageModal>>>('createVersionModal')

const editLoaders = () => {
	createVersionModal?.value?.setStage(
		createVersionModal?.value?.stages.findIndex((stage) => stage.title === 'Edit loaders'),
	)
}
const editVersions = () => {
	createVersionModal?.value?.setStage(
		createVersionModal?.value?.stages.findIndex((stage) => stage.title === 'Edit MC versions'),
	)
}
</script>
