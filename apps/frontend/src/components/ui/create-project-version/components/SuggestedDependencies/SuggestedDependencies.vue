<template>
	<div v-if="visibleDependencies.length" class="flex flex-col gap-4">
		<span class="font-semibold text-contrast">Suggested dependencies</span>
		<div class="flex flex-col gap-2">
			<template v-for="(dependency, index) in visibleDependencies">
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
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'

import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

import SuggestedDependency from './SuggestedDependency.vue'

export interface SuggestedDependency extends Labrinth.Versions.v3.Dependency {
	icon?: string
	name?: string
	versionName?: string
}

const props = defineProps<{
	suggestedDependencies: SuggestedDependency[]
}>()

const { draftVersion } = injectManageVersionContext()

const visibleDependencies = computed<SuggestedDependency[]>(() =>
	props.suggestedDependencies
		.filter(
			(dep) =>
				!draftVersion.value.dependencies?.some(
					(d) => d.project_id === dep.project_id && d.version_id === dep.version_id,
				),
		)
		.sort((a, b) => (a.name || '').localeCompare(b.name || '')),
)

const emit = defineEmits<{
	(e: 'onAddSuggestion', dependency: Labrinth.Versions.v3.Dependency): void
}>()

function handleAddSuggestion(dependency: Labrinth.Versions.v3.Dependency) {
	emit('onAddSuggestion', dependency)
}
</script>
