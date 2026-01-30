<template>
	<div v-if="visibleSuggestedDependencies.length" class="flex flex-col gap-2">
		<template v-for="(dependency, index) in visibleSuggestedDependencies">
			<SuggestedDependency
				v-if="dependency"
				:key="index"
				:project-id="dependency.project_id"
				:name="dependency.name"
				:icon="dependency.icon"
				:dependency-type="dependency.dependency_type"
				:version-name="dependency.versionName"
				@on-add-suggestion="
					() =>
						handleAddSuggestion({
							dependency_type: dependency.dependency_type,
							project_id: dependency.project_id,
							version_id: dependency.version_id,
						})
				"
			/>
		</template>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'

import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

import SuggestedDependency from './SuggestedDependency.vue'

const { visibleSuggestedDependencies } = injectManageVersionContext()

const emit = defineEmits<{
	(e: 'onAddSuggestion', dependency: Labrinth.Versions.v3.Dependency): void
}>()

function handleAddSuggestion(dependency: Labrinth.Versions.v3.Dependency) {
	emit('onAddSuggestion', dependency)
}
</script>
