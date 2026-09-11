<template>
	<!-- Detail view: a selected version replaces the list (with a way back) -->

	<div v-if="selectedVersion" class="flex flex-col gap-3">
		<VersionPage
			:version="selectedVersion"
			:on-back-route="() => layout.setSelectedVersion(null)"
		/>
	</div>
	<!-- List view -->
	<div v-else>
		<div>
			<ProjectSidebarCompatibility
				:project="project"
				:tags="tags"
				:project-v3="projectV3"
				:compact-mode="true"
				class="flex-card-reduced pb-3"
			/>
		</div>
		<Suspense>
			<VersionsPage
				:version-row-click="
					(version) => {
						layout.setSelectedVersion(version.id)
					}
				"
			/>
			<template #fallback>
				<div class="flex items-center justify-center gap-2 py-12 text-secondary">
					<SpinnerIcon class="size-5 animate-spin" /> Loading versions…
				</div>
			</template>
		</Suspense>
	</div>
</template>

<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { injectProjectPageContext, ProjectSidebarCompatibility } from '@modrinth/ui'
import { computed, defineAsyncComponent, onMounted } from 'vue'

import VersionPage from '~/pages/[type]/[project]/version/[version].vue'
import { useModerationReviewLayout } from '~/services/moderation/review-layout'

const tags = useGeneratedState()

const VersionsPage = defineAsyncComponent(() => import('~/pages/[type]/[project]/versions.vue'))

const layout = useModerationReviewLayout()
const { projectV2: project, projectV3, versions, loadDependencies } = injectProjectPageContext()

onMounted(() => loadDependencies())

const selectedVersion = computed(() =>
	layout.selectedVersionId.value
		? versions.value?.find((v) => v.id === layout.selectedVersionId.value)
		: undefined,
)
</script>
