<template>
	<div class="grid gap-6">
		<div class="flex flex-col gap-4">
			<span class="font-semibold text-contrast">Add dependency</span>
			<div class="grid gap-3 rounded-2xl border border-solid border-surface-5 p-4">
				<div class="grid gap-2.5">
					<span class="font-semibold text-contrast">Project <span class="text-red">*</span></span>
					<ModSelect v-model="newDependencyProjectId" />
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
				</template>
			</div>
			<ButtonStyled>
				<button
					class="self-start"
					:disabled="!newDependencyProjectId"
					@click="
						() =>
							addDependency({
								project_id: newDependencyProjectId,
								version_id: newDependencyVersionId || undefined,
								dependency_type: newDependencyType,
							})
					"
				>
					Add Dependency
				</button>
			</ButtonStyled>
		</div>

		<div v-if="addedDependencies.length" class="flex flex-col gap-4">
			<span class="font-semibold text-contrast">Added dependencies</span>
			<div class="5 flex flex-col gap-2">
				<template v-for="(dependency, index) in addedDependencies">
					<AddedDependencyRow
						v-if="dependency"
						:key="index"
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
import { injectModrinthClient, injectNotificationManager } from '@modrinth/ui'
import ButtonStyled from '@modrinth/ui/src/components/base/ButtonStyled.vue'
import type { DropdownOption } from '@modrinth/ui/src/components/base/Combobox.vue'
import Combobox from '@modrinth/ui/src/components/base/Combobox.vue'

import ModSelect from '~/components/ui/create-project-version/components/ModSelect.vue'
import { useManageVersion } from '~/composables/versions/manage-version'

import AddedDependencyRow from '../components/AddedDependencyRow.vue'

const { addNotification } = injectNotificationManager()

const errorNotification = (err: any) => {
	addNotification({
		title: 'An error occurred',
		text: err.data ? err.data.description : err,
		type: 'error',
	})
}

const newDependencyProjectId = ref<string>()
const newDependencyType = ref<Labrinth.Versions.v3.DependencyType>('required')
const newDependencyVersionId = ref<string | null>(null)

const newDependencyVersions = ref<DropdownOption<string>[]>([])

// reset to defaults when select different project
watch(newDependencyProjectId, async () => {
	newDependencyVersionId.value = null
	newDependencyType.value = 'required'

	if (!newDependencyProjectId.value) {
		newDependencyVersions.value = []
	} else {
		try {
			const versions = await client.labrinth.versions_v3.getProjectVersions(
				newDependencyProjectId.value,
			)
			newDependencyVersions.value = versions.map((version) => ({
				label: version.name,
				value: version.id,
			}))
		} catch (error: any) {
			errorNotification(error)
		}
	}
})

const { draftVersion } = useManageVersion()
const client = injectModrinthClient()

const dependencyProjects = ref<Record<string, Labrinth.Projects.v3.Project>>({})
const dependencyVersions = ref<Record<string, Labrinth.Versions.v3.Version>>({})

watch(
	draftVersion,
	async (draftVersion) => {
		const deps = draftVersion.dependencies || []

		for (const dep of deps) {
			if (dep?.project_id && !dependencyProjects.value[dep.project_id]) {
				try {
					const project = await client.labrinth.projects_v3.get(dep.project_id)
					dependencyProjects.value[dep.project_id] = project
				} catch (error: any) {
					errorNotification(error)
				}
			}

			if (dep?.version_id && !dependencyVersions.value[dep.version_id]) {
				try {
					const version = await client.labrinth.versions_v3.getVersion(dep.version_id)
					dependencyVersions.value[dep.version_id] = version
				} catch (error: any) {
					errorNotification(error)
				}
			}
		}
	},
	{ immediate: true, deep: true },
)

const addedDependencies = computed(() =>
	(draftVersion.value.dependencies || []).map((dep) => {
		if (!dep.project_id) return null

		const dependencyProject = dependencyProjects.value[dep.project_id]
		const versionName = dependencyVersions.value[dep.version_id || '']?.name ?? ''

		if (!dependencyProject) return null

		return {
			projectId: dep.project_id,
			name: dependencyProject?.name || 'Unknown Project',
			icon: dependencyProject?.icon_url,
			dependencyType: 'required' as const,
			versionName,
		}
	}),
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

	draftVersion.value.dependencies.push({
		project_id: dependency.project_id,
		version_id: dependency.version_id,
		dependency_type: dependency.dependency_type,
	})

	newDependencyProjectId.value = undefined
}

const removeDependency = (index: number) => {
	if (!draftVersion.value.dependencies) return
	draftVersion.value.dependencies.splice(index, 1)
}
</script>
