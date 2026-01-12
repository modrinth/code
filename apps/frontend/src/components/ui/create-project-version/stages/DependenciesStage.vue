<template>
	<div class="flex w-full max-w-full flex-col gap-6">
		<div class="flex flex-col gap-4">
			<span class="font-semibold text-contrast">Add dependency</span>
			<div class="flex flex-col gap-3 rounded-2xl border border-solid border-surface-5 p-4">
				<div class="grid gap-2.5">
					<span class="font-semibold text-contrast">Project</span>
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

					<ButtonStyled color="green">
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

		<div v-if="visibleSuggestedDependencies.length" class="flex flex-col gap-4">
			<span class="font-semibold text-contrast">Suggested dependencies</span>
			<SuggestedDependencies @on-add-suggestion="handleAddSuggestedDependency" />
		</div>

		<div v-if="addedDependencies.length" class="flex flex-col gap-4">
			<span class="font-semibold text-contrast">Added dependencies</span>
			<DependenciesList />
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
} from '@modrinth/ui'
import type { ComboboxOption } from '@modrinth/ui/src/components/base/Combobox.vue'

import DependencySelect from '~/components/ui/create-project-version/components/DependencySelect.vue'
import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

import DependenciesList from '../components/DependenciesList.vue'
import SuggestedDependencies from '../components/SuggestedDependencies/SuggestedDependencies.vue'

const { addNotification } = injectNotificationManager()
const { labrinth } = injectModrinthClient()

const {
	draftVersion,
	dependencyProjects,
	dependencyVersions,
	projectsFetchLoading,
	visibleSuggestedDependencies,
} = injectManageVersionContext()

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

const newDependencyVersions = ref<ComboboxOption<string>[]>([])

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

	const alreadyAdded = draftVersion.value.dependencies.some((existing) => {
		if (existing.project_id !== dependency.project_id) return false
		if (!existing.version_id && !dependency.version_id) return true
		return existing.version_id === dependency.version_id
	})

	if (alreadyAdded) {
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

const handleAddSuggestedDependency = (dependency: Labrinth.Versions.v3.Dependency) => {
	draftVersion.value.dependencies?.push({
		project_id: dependency.project_id,
		version_id: dependency.version_id,
		dependency_type: dependency.dependency_type,
	})
}
</script>
