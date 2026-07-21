<template>
	<Tabs
		v-if="editingVersion"
		value="metadata"
		:tabs="editTabs"
		class="mb-3 border border-solid border-surface-5 !shadow-none !drop-shadow-none"
		@change="setEditTab"
	/>
	<div class="flex flex-col gap-6">
		<div v-if="!editingVersion" class="flex flex-col gap-1">
			<div class="flex items-center justify-between">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.uploadedFiles) }}</span>

				<ButtonStyled type="transparent" size="standard">
					<button @click="editFiles">
						<EditIcon />
						{{ formatMessage(messages.editButton) }}
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
					{{
						usingDetectedLoaders
							? formatMessage(messages.detectedLoaders)
							: formatMessage(messages.loaders)
					}}
				</span>

				<ButtonStyled type="transparent" size="standard">
					<button
						v-tooltip="
							isModpack
								? formatMessage(messages.modpackLoadersTooltip)
								: isResourcePack
									? formatMessage(messages.resourcePackLoadersTooltip)
									: undefined
						"
						:disabled="isModpack || isResourcePack"
						@click="editLoaders"
					>
						<EditIcon />
						{{ formatMessage(messages.editButton) }}
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
							<component :is="getLoaderIcon(loader.name)" v-if="getLoaderIcon(loader.name)" />
							<FormattedTag :tag="loader.name" enforce-type="loader" />
						</TagItem>
					</template>

					<TagItem
						v-if="!draftVersionLoaders.length && projectType === 'modpack'"
						class="border !border-solid border-surface-5 hover:no-underline"
					>
						{{ formatMessage(messages.noModLoader) }}
					</TagItem>
					<span v-else-if="!draftVersionLoaders.length">{{
						formatMessage(messages.noLoadersSelected)
					}}</span>
				</div>
			</div>
		</div>

		<div class="flex flex-col gap-1">
			<div class="flex items-center justify-between">
				<span class="font-semibold text-contrast">
					{{
						usingDetectedVersions
							? formatMessage(messages.detectedVersions)
							: formatMessage(messages.versions)
					}}
				</span>

				<ButtonStyled type="transparent" size="standard">
					<button
						v-tooltip="isModpack ? formatMessage(messages.modpackVersionsTooltip) : undefined"
						:disabled="isModpack"
						@click="editVersions"
					>
						<EditIcon />
						{{ formatMessage(messages.editButton) }}
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

					<span v-if="!draftVersion.game_versions.length">{{
						formatMessage(messages.noVersionsSelected)
					}}</span>
				</div>
			</div>
		</div>

		<template v-if="!noEnvironmentProject">
			<div class="flex flex-col gap-1">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-2">
						<span class="font-semibold text-contrast">{{
							formatMessage(messages.environment)
						}}</span>
						<UnknownIcon v-tooltip="formatMessage(messages.prefilledEnvironmentTooltip)" />
					</div>

					<ButtonStyled type="transparent" size="standard">
						<button @click="editEnvironment">
							<EditIcon />
							{{ formatMessage(messages.editButton) }}
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

					<span v-else class="text-sm font-medium">{{
						formatMessage(messages.noEnvironmentSet)
					}}</span>
				</div>
			</div>
		</template>

		<template v-if="!noDependenciesProject">
			<div class="flex flex-col gap-2.5">
				<div class="flex flex-col gap-1">
					<div class="flex items-center justify-between">
						<span class="font-semibold text-contrast">{{
							formatMessage(messages.dependencies)
						}}</span>

						<ButtonStyled type="transparent" size="standard">
							<button @click="addDependency">
								<PlusIcon />
								{{ formatMessage(messages.addDependency) }}
							</button>
						</ButtonStyled>
					</div>

					<div v-if="draftVersion.dependencies?.length" class="flex flex-col gap-4">
						<DependenciesList />
					</div>
					<div v-else class="flex flex-col gap-1.5 gap-y-4 rounded-xl bg-surface-2 p-3 py-4">
						<span class="text-sm font-medium">{{
							formatMessage(messages.noDependenciesAdded)
						}}</span>
					</div>
				</div>

				<div v-if="visibleSuggestedDependencies.length" class="flex flex-col gap-2.5">
					<div class="flex items-center justify-between">
						<span class="font-medium">{{ formatMessage(messages.suggested) }}</span>
					</div>
					<SuggestedDependencies @on-add-suggestion="handleAddSuggestedDependency" />
				</div>
			</div>
		</template>
	</div>
</template>

<script lang="ts" setup>
import type { Labrinth } from '@modrinth/api-client'
import { EditIcon, getLoaderIcon, PlusIcon, UnknownIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	defineMessages,
	ENVIRONMENTS_COPY,
	FormattedTag,
	injectProjectPageContext,
	Tabs,
	type TabsTab,
	TagItem,
	useVIntl,
} from '@modrinth/ui'

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

const editTabs = computed<TabsTab[]>(() => [
	{ label: formatMessage(messages.metadataTab), value: 'metadata' },
	{ label: formatMessage(messages.detailsTab), value: 'add-details' },
	{ label: formatMessage(messages.filesTab), value: 'add-files' },
])

function setEditTab(tab: TabsTab) {
	modal.value?.setStage(tab.value)
}

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
const addDependency = () => {
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

const messages = defineMessages({
	uploadedFiles: {
		id: 'create-project-version.create-modal.stage.metadata.uploaded-files',
		defaultMessage: 'Uploaded files',
	},
	editButton: {
		id: 'create-project-version.create-modal.stage.metadata.edit-button',
		defaultMessage: 'Edit',
	},
	detectedLoaders: {
		id: 'create-project-version.create-modal.stage.metadata.detected-loaders',
		defaultMessage: 'Detected loaders',
	},
	loaders: {
		id: 'create-project-version.create-modal.stage.metadata.loaders',
		defaultMessage: 'Loaders',
	},
	modpackLoadersTooltip: {
		id: 'create-project-version.create-modal.stage.metadata.modpack-loaders-tooltip',
		defaultMessage: 'Modpack loaders cannot be edited',
	},
	resourcePackLoadersTooltip: {
		id: 'create-project-version.create-modal.stage.metadata.resource-pack-loaders-tooltip',
		defaultMessage: 'Resource pack loaders cannot be edited',
	},
	noModLoader: {
		id: 'create-project-version.create-modal.stage.metadata.no-mod-loader',
		defaultMessage: 'No mod loader',
	},
	noLoadersSelected: {
		id: 'create-project-version.create-modal.stage.metadata.no-loaders-selected',
		defaultMessage: 'No loaders selected.',
	},
	detectedVersions: {
		id: 'create-project-version.create-modal.stage.metadata.detected-versions',
		defaultMessage: 'Detected versions',
	},
	versions: {
		id: 'create-project-version.create-modal.stage.metadata.versions',
		defaultMessage: 'Versions',
	},
	modpackVersionsTooltip: {
		id: 'create-project-version.create-modal.stage.metadata.modpack-versions-tooltip',
		defaultMessage: 'Modpack versions cannot be edited',
	},
	noVersionsSelected: {
		id: 'create-project-version.create-modal.stage.metadata.no-versions-selected',
		defaultMessage: 'No versions selected.',
	},
	environment: {
		id: 'create-project-version.create-modal.stage.metadata.environment',
		defaultMessage: 'Environment',
	},
	prefilledEnvironmentTooltip: {
		id: 'create-project-version.create-modal.stage.metadata.prefilled-environment-tooltip',
		defaultMessage: 'Pre-filled from a previous similar version',
	},
	noEnvironmentSet: {
		id: 'create-project-version.create-modal.stage.metadata.no-environment-set',
		defaultMessage: 'No environment has been set.',
	},
	dependencies: {
		id: 'create-project-version.create-modal.stage.metadata.dependencies',
		defaultMessage: 'Dependencies',
	},
	addDependency: {
		id: 'create-project-version.create-modal.stage.metadata.add-dependency',
		defaultMessage: 'Add dependency',
	},
	noDependenciesAdded: {
		id: 'create-project-version.create-modal.stage.metadata.no-dependencies-added',
		defaultMessage: 'No dependencies added.',
	},
	suggested: {
		id: 'create-project-version.create-modal.stage.metadata.suggested',
		defaultMessage: 'Suggested',
	},
	metadataTab: {
		id: 'create-project-version.create-modal.stage.metadata.metadata-tab',
		defaultMessage: 'Metadata',
	},
	detailsTab: {
		id: 'create-project-version.create-modal.stage.metadata.details-tab',
		defaultMessage: 'Details',
	},
	filesTab: {
		id: 'create-project-version.create-modal.stage.metadata.files-tab',
		defaultMessage: 'Files',
	},
})

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
