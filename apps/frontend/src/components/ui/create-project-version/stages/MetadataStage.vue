<template>
	<div class="flex flex-col gap-6">
		<div v-if="!editingVersion" class="flex flex-col gap-1">
			<div class="flex items-center justify-between">
				<span class="font-semibold text-contrast"> Uploaded files </span>

				<ButtonStyled type="transparent" size="standard">
					<button @click="editFiles">
						<EditIcon />
						Edit
					</button>
				</ButtonStyled>
			</div>
			<div class="flex flex-col gap-2.5">
				<ViewOnlyFileRow
					v-if="primaryFile"
					:key="primaryFile.name"
					:name="primaryFile.name"
					:is-primary="true"
				/>
				<ViewOnlyFileRow
					v-for="file in supplementaryNewFiles"
					:key="file.file.name"
					:name="file.file.name"
					:file-type="file.fileType"
				/>
				<ViewOnlyFileRow
					v-for="file in supplementaryExistingFiles"
					:key="file.filename"
					:name="file.filename"
					:file-type="file.file_type"
				/>
			</div>
		</div>

		<div class="flex flex-col gap-1">
			<div class="flex items-center justify-between">
				<span class="font-semibold text-contrast">
					{{ usingDetectedLoaders ? 'Detected loaders' : 'Loaders' }}
				</span>

				<ButtonStyled type="transparent" size="standard">
					<button
						v-tooltip="
							isModpack
								? 'Modpack loaders cannot be edited'
								: isResourcePack
									? 'Resource pack loaders cannot be edited'
									: undefined
						"
						:disabled="isModpack || isResourcePack"
						@click="editLoaders"
					>
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
						v-for="loader in draftVersionLoaders.map((selectedLoader) =>
							loaders.find((loader) => selectedLoader === loader.name),
						)"
					>
						<TagItem
							v-if="loader"
							:key="`loader-${loader.name}`"
							class="border !border-solid border-surface-5 hover:no-underline"
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

		<div class="flex flex-col gap-1">
			<div class="flex items-center justify-between">
				<span class="font-semibold text-contrast">
					{{ usingDetectedVersions ? 'Detected versions' : 'Versions' }}
				</span>

				<ButtonStyled type="transparent" size="standard">
					<button
						v-tooltip="isModpack ? 'Modpack versions cannot be edited' : undefined"
						:disabled="isModpack"
						@click="editVersions"
					>
						<EditIcon />
						Edit
					</button>
				</ButtonStyled>
			</div>

			<div
				class="flex max-h-56 flex-col gap-1.5 gap-y-4 overflow-y-auto rounded-xl border border-solid border-surface-5 p-3 py-4"
			>
				<div class="flex flex-wrap gap-2">
					<TagItem
						v-for="version in draftVersion.game_versions"
						:key="version"
						class="border !border-solid border-surface-5 hover:no-underline"
					>
						{{ version }}
					</TagItem>

					<span v-if="!draftVersion.game_versions.length">No versions selected.</span>
				</div>
			</div>
		</div>

		<template v-if="!noEnvironmentProject">
			<div class="flex flex-col gap-1">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-2">
						<span class="font-semibold text-contrast"> Environment </span>
						<UnknownIcon v-tooltip="'Pre-filled from a previous similar version'" />
					</div>

					<ButtonStyled type="transparent" size="standard">
						<button @click="editEnvironment">
							<EditIcon />
							Edit
						</button>
					</ButtonStyled>
				</div>

				<div class="flex flex-col gap-1.5 gap-y-4 rounded-xl bg-surface-2 p-3 py-4">
					<div v-if="draftVersion.environment" class="flex flex-col gap-1">
						<div class="font-semibold text-contrast">
							{{ environmentCopy.title }}
						</div>
						<div class="text-sm font-medium">{{ environmentCopy.description }}</div>
					</div>

					<span v-else class="text-sm font-medium">No environment has been set.</span>
				</div>
			</div>
		</template>

		<template v-if="!noDependenciesProject">
			<div v-if="visibleSuggestedDependencies.length" class="flex flex-col gap-1">
				<div class="flex items-center justify-between">
					<span class="font-semibold text-contrast"> Suggested dependencies </span>

					<ButtonStyled type="transparent" size="standard">
						<button @click="editDependencies">
							<EditIcon />
							Edit
						</button>
					</ButtonStyled>
				</div>
				<SuggestedDependencies @on-add-suggestion="handleAddSuggestedDependency" />
			</div>

			<div
				v-if="!visibleSuggestedDependencies.length || draftVersion.dependencies?.length"
				class="flex flex-col gap-1"
			>
				<div class="flex items-center justify-between">
					<span class="font-semibold text-contrast"> Dependencies </span>

					<ButtonStyled type="transparent" size="standard">
						<button @click="editDependencies">
							<EditIcon />
							Edit
						</button>
					</ButtonStyled>
				</div>

				<div v-if="draftVersion.dependencies?.length" class="flex flex-col gap-4">
					<DependenciesList disable-remove />
				</div>
				<div v-else class="flex flex-col gap-1.5 gap-y-4 rounded-xl bg-surface-2 p-3 py-4">
					<span class="text-sm font-medium">No dependencies added.</span>
				</div>
			</div>
		</template>
	</div>
</template>

<script lang="ts" setup>
import type { Labrinth } from '@modrinth/api-client'
import { EditIcon, UnknownIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	defineMessages,
	ENVIRONMENTS_COPY,
	injectProjectPageContext,
	TagItem,
	useVIntl,
} from '@modrinth/ui'
import { formatCategory } from '@modrinth/utils'

import { useGeneratedState } from '~/composables/generated'
import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

import DependenciesList from '../components/DependenciesList.vue'
import SuggestedDependencies from '../components/SuggestedDependencies/SuggestedDependencies.vue'
import ViewOnlyFileRow from '../components/ViewOnlyFileRow.vue'

const {
	draftVersion,
	inferredVersionData,
	projectType,
	noEnvironmentProject,
	noDependenciesProject,
	modal,
	filesToAdd,
	editingVersion,
	visibleSuggestedDependencies,
} = injectManageVersionContext()

const { projectV2 } = injectProjectPageContext()

const generatedState = useGeneratedState()
const loaders = computed(() => generatedState.value.loaders)
const isModpack = computed(() => projectType.value === 'modpack')
const isResourcePack = computed(
	() =>
		projectType.value === 'resourcepack' &&
		(projectV2.value?.project_type === 'resourcepack' ||
			projectV2.value?.project_type === 'project'),
)

const draftVersionLoaders = computed(() =>
	[
		...new Set([...draftVersion.value.loaders, ...(draftVersion.value.mrpack_loaders ?? [])]),
	].filter((loader) => loader !== 'mrpack'),
)

const editLoaders = () => {
	modal.value?.setStage('from-details-loaders')
}
const editVersions = () => {
	modal.value?.setStage('from-details-mc-versions')
}
const editEnvironment = () => {
	modal.value?.setStage('from-details-environment')
}
const editFiles = () => {
	modal.value?.setStage('from-details-files')
}
const editDependencies = () => {
	modal.value?.setStage('from-details-dependencies')
}

const usingDetectedVersions = computed(() => {
	if (!inferredVersionData.value?.game_versions) return false

	const versionsMatch =
		draftVersion.value.game_versions.length === inferredVersionData.value.game_versions.length &&
		draftVersion.value.game_versions.every((version) =>
			inferredVersionData.value?.game_versions?.includes(version),
		)

	return versionsMatch
})

const usingDetectedLoaders = computed(() => {
	if (!inferredVersionData.value?.loaders) return false

	const loadersMatch =
		draftVersion.value.loaders.length === inferredVersionData.value.loaders.length &&
		draftVersion.value.loaders.every((loader) =>
			inferredVersionData.value?.loaders?.includes(loader),
		)

	return loadersMatch
})

interface PrimaryFile {
	name: string
	fileType?: string
	existing?: boolean
}

const primaryFile = computed<PrimaryFile | null>(() => {
	const existingPrimaryFile = draftVersion.value.existing_files?.[0]
	if (existingPrimaryFile) {
		return {
			name: existingPrimaryFile.filename,
			fileType: existingPrimaryFile.file_type,
			existing: true,
		}
	}

	const addedPrimaryFile = filesToAdd.value[0]
	if (addedPrimaryFile) {
		return {
			name: addedPrimaryFile.file.name,
			fileType: addedPrimaryFile.fileType,
			existing: false,
		}
	}

	return null
})

const supplementaryNewFiles = computed(() => {
	if (primaryFile.value?.existing) {
		return filesToAdd.value
	} else {
		return filesToAdd.value.slice(1)
	}
})

const supplementaryExistingFiles = computed(() => {
	if (primaryFile.value?.existing) {
		return draftVersion.value.existing_files?.slice(1)
	} else {
		return draftVersion.value.existing_files
	}
})

const { formatMessage } = useVIntl()

const noEnvironmentMessage = defineMessages({
	title: {
		id: 'version.environment.none.title',
		defaultMessage: 'No environment set',
	},
	description: {
		id: 'version.environment.none.description',
		defaultMessage: 'The environment for this version has not been specified.',
	},
})

const unknownEnvironmentMessage = defineMessages({
	title: {
		id: 'version.environment.unknown.title',
		defaultMessage: 'Unknown environment',
	},
	description: {
		id: 'version.environment.unknown.description',
		defaultMessage: 'The environment: "{environment}" is not recognized.',
	},
})

const environmentCopy = computed(() => {
	if (!draftVersion.value.environment) {
		return {
			title: formatMessage(noEnvironmentMessage.title),
			description: formatMessage(noEnvironmentMessage.description),
		}
	}

	const envCopy = ENVIRONMENTS_COPY[draftVersion.value.environment]
	if (envCopy) {
		return {
			title: formatMessage(envCopy.title),
			description: formatMessage(envCopy.description),
		}
	}

	return {
		title: formatMessage(unknownEnvironmentMessage.title),
		description: formatMessage(unknownEnvironmentMessage.description, {
			environment: draftVersion.value.environment,
		}),
	}
})

const handleAddSuggestedDependency = (dependency: Labrinth.Versions.v3.Dependency) => {
	if (!draftVersion.value.dependencies) draftVersion.value.dependencies = []
	draftVersion.value.dependencies.push({
		project_id: dependency.project_id,
		version_id: dependency.version_id,
		dependency_type: dependency.dependency_type,
	})
}
</script>
