<template>
	<div v-if="addedDependencies.length" class="5 flex flex-col gap-2">
		<template v-for="(dependency, index) in addedDependencies">
			<AddedDependencyRow
				v-if="dependency"
				:key="index"
				:project-id="dependency.projectId"
				:name="dependency.name"
				:icon="dependency.icon"
				:dependency-type="dependency.dependencyType"
				:version-name="dependency.versionName"
				:hide-remove="disableRemove"
				@remove="() => removeDependency(index)"
			/>
		</template>
		<span v-if="!addedDependencies.length"> No dependencies added. </span>
	</div>
</template>

<script setup lang="ts">
import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

import AddedDependencyRow from './AddedDependencyRow.vue'

const { disableRemove } = defineProps<{
	disableRemove?: boolean
}>()

const { draftVersion, dependencyProjects, dependencyVersions, projectsFetchLoading } =
	injectManageVersionContext()

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

const removeDependency = (index: number) => {
	if (!draftVersion.value.dependencies) return
	draftVersion.value.dependencies.splice(index, 1)
}
</script>
