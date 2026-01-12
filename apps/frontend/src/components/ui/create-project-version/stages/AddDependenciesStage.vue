<template>
	<div class="flex w-full max-w-full flex-col gap-6 sm:w-[512px]">
		<div class="flex flex-col gap-4">
			<span class="font-semibold text-contrast">Add dependency</span>
			<div class="flex flex-col gap-3 rounded-2xl border border-solid border-surface-5 p-4">
				<div class="grid gap-2.5">
					<span class="font-semibold text-contrast">Project <span class="text-red">*</span></span>
					<DependencySelect v-model="newDependencyProjectId" />
				</div>

				<template v-if="newDependencyProjectId">
					<div class="grid gap-2.5">
						<span class="font-semibold text-contrast"> Version </span>
						<Combobox
							v-model="newDependencyVersionId"
							placeholder="Select version"
							:options="[{ label: 'Any version', value: null }, ...newDependencyVersions]"
							:searchable="true"
						/>
					</div>

					<div class="grid gap-2.5">
						<span class="font-semibold text-contrast"> Dependency relation </span>
						<Combobox
							v-model="newDependencyType"
							placeholder="Select dependency type"
							:options="[
								{ label: 'Required', value: 'required' },
								{ label: 'Optional', value: 'optional' },
								{ label: 'Incompatible', value: 'incompatible' },
								{ label: 'Embedded', value: 'embedded' },
							]"
						/>
					</div>

					<ButtonStyled>
						<button
							class="self-start"
							:disabled="!newDependencyProjectId"
							@click="
								() =>
									addDependency(
										toRaw({
											project_id: newDependencyProjectId,
											version_id: newDependencyVersionId || undefined,
											dependency_type: newDependencyType,
										}),
									)
							"
						>
							Add Dependency
						</button>
					</ButtonStyled>
				</template>
			</div>
		</div>

		<SuggestedDependencies
			:suggested-dependencies="suggestedDependencies"
			@on-add-suggestion="handleAddSuggestedDependency"
		/>

		<div v-if="addedDependencies.length" class="flex flex-col gap-4">
			<span class="font-semibold text-contrast">Added dependencies</span>
			<div class="5 flex flex-col gap-2">
				<template v-for="(dependency, index) in addedDependencies">
					<AddedDependencyRow
						v-if="dependency"
						:key="index"
						:project-id="dependency.projectId"
						:name="dependency.name"
						:icon="dependency.icon"
						:dependency-type="dependency.dependencyType"
						:version-name="dependency.versionName"
						@remove="() => removeDependency(index)"
					/>
				</template>
				<span v-if="!addedDependencies.length"> No dependencies added. </span>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
import type { Labrinth } from '@modrinth/api-client'
import {
	ButtonStyled,
	Combobox,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
} from '@modrinth/ui'
import type { DropdownOption } from '@modrinth/ui/src/components/base/Combobox.vue'

import DependencySelect from '~/components/ui/create-project-version/components/DependencySelect.vue'
import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

import AddedDependencyRow from '../components/AddedDependencyRow.vue'
import SuggestedDependencies from '../components/SuggestedDependencies/SuggestedDependencies.vue'

const { addNotification } = injectNotificationManager()
const { labrinth } = injectModrinthClient()

const errorNotification = (err: any) => {
	addNotification({
		title: 'An error occurred',
		text: err.data ? err.data.description : err,
		type: 'error',
	})
}

const newDependencyProjectId = ref<string>()
const newDependencyType = ref<Labrinth.Versions.v2.DependencyType>('required')
const newDependencyVersionId = ref<string | null>(null)

const newDependencyVersions = ref<DropdownOption<string>[]>([])

const projectsFetchLoading = ref(false)
const suggestedDependencies = ref<
	Array<Labrinth.Versions.v3.Dependency & { name?: string; icon?: string; versionName?: string }>
>([])

// reset to defaults when select different project
watch(newDependencyProjectId, async () => {
	newDependencyVersionId.value = null
	newDependencyType.value = 'required'

	if (!newDependencyProjectId.value) {
		newDependencyVersions.value = []
	} else {
		try {
			const versions = await labrinth.versions_v3.getProjectVersions(newDependencyProjectId.value)
			newDependencyVersions.value = versions.map((version) => ({
				label: version.name,
				value: version.id,
			}))
		} catch (error: any) {
			errorNotification(error)
		}
	}
})

const { draftVersion, dependencyProjects, dependencyVersions, getProject, getVersion } =
	injectManageVersionContext()
const { projectV2: project } = injectProjectPageContext()

const getSuggestedDependencies = async () => {
	try {
		suggestedDependencies.value = []

		if (!draftVersion.value.game_versions?.length || !draftVersion.value.loaders?.length) {
			return
		}

		try {
			const versions = await labrinth.versions_v3.getProjectVersions(project.value.id, {
				loaders: draftVersion.value.loaders,
			})

			// Get the most recent matching version and extract its dependencies
			if (versions.length > 0) {
				const mostRecentVersion = versions[0]
				for (const dep of mostRecentVersion.dependencies) {
					suggestedDependencies.value.push({
						project_id: dep.project_id,
						version_id: dep.version_id,
						dependency_type: dep.dependency_type,
						file_name: dep.file_name,
					})
				}
			}
		} catch (error: any) {
			console.error(`Failed to get versions for project ${project.value.id}:`, error)
		}

		for (const dep of suggestedDependencies.value) {
			try {
				if (dep.project_id) {
					const proj = await getProject(dep.project_id)
					dep.name = proj.name
					dep.icon = proj.icon_url
				}

				if (dep.version_id) {
					const version = await getVersion(dep.version_id)
					dep.versionName = version.name
				}
			} catch (error: any) {
				console.error(`Failed to fetch project/version data for dependency:`, error)
			}
		}
	} catch (error: any) {
		errorNotification(error)
	}
}

onMounted(() => {
	getSuggestedDependencies()
})

watch(
	draftVersion,
	async (draftVersion) => {
		const deps = draftVersion.dependencies || []

		for (const dep of deps) {
			if (dep?.project_id) {
				try {
					await getProject(dep.project_id)
				} catch (error: any) {
					errorNotification(error)
				}
			}

			if (dep?.version_id) {
				try {
					await getVersion(dep.version_id)
				} catch (error: any) {
					errorNotification(error)
				}
			}
		}
		projectsFetchLoading.value = false
	},
	{ immediate: true, deep: true },
)

const addedDependencies = computed(() =>
	(draftVersion.value.dependencies || [])
		.map((dep) => {
			if (!dep.project_id) return null

			const dependencyProject = dependencyProjects.value[dep.project_id]
			const versionName = dependencyVersions.value[dep.version_id || '']?.name ?? ''

			if (!dependencyProject && projectsFetchLoading.value) return null

			return {
				projectId: dep.project_id,
				name: dependencyProject?.name,
				icon: dependencyProject?.icon_url,
				dependencyType: dep.dependency_type,
				versionName,
			}
		})
		.filter(Boolean),
)

const addDependency = (dependency: Labrinth.Versions.v3.Dependency) => {
	if (!draftVersion.value.dependencies) draftVersion.value.dependencies = []

	// already added
	if (
		draftVersion.value.dependencies.find(
			(d) => d.project_id === dependency.project_id && d.version_id === dependency.version_id,
		)
	) {
		addNotification({
			title: 'Dependency already added',
			text: 'You cannot add the same dependency twice.',
			type: 'error',
		})
		return
	}

	projectsFetchLoading.value = true
	draftVersion.value.dependencies.push(dependency)
	newDependencyProjectId.value = undefined
}

const removeDependency = (index: number) => {
	if (!draftVersion.value.dependencies) return
	draftVersion.value.dependencies.splice(index, 1)
}

const handleAddSuggestedDependency = (dependency: Labrinth.Versions.v3.Dependency) => {
	draftVersion.value.dependencies?.push({
		project_id: dependency.project_id,
		version_id: dependency.version_id,
		dependency_type: dependency.dependency_type,
	})
}
</script>
