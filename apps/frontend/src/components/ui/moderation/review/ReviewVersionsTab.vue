<template>
	<!-- Detail view: a selected version replaces the list (with a way back) -->

	<div v-if="selectedVersion" class="flex flex-col gap-3">
		<VersionPage
			:version="selectedVersion"
			:on-back-route="() => layout.setSelectedVersion(null)"
		/>
	</div>
	<!-- List view -->
	<div v-else >
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
			<VersionsPage :versionRowClick="(version) => {
				layout.setSelectedVersion(version.id)
			}"/>
			<template #fallback>
				<div class="flex items-center justify-center gap-2 py-12 text-secondary">
					<SpinnerIcon class="size-5 animate-spin" /> Loading versions…
				</div>
			</template>
		</Suspense>
	</div>
</template>

<script setup lang="ts">
import { LeftArrowIcon, SpinnerIcon } from '@modrinth/assets'
import { injectProjectPageContext, ProjectSidebarCompatibility} from '@modrinth/ui'
import { computed, defineAsyncComponent, handleError, onMounted, ref } from 'vue'

import { useModerationReviewLayout } from '~/services/moderation/review-layout'
import VersionPage from '~/pages/[type]/[project]/version/[version].vue'

const tags = useGeneratedState()

const VersionsPage = defineAsyncComponent(() => import('~/pages/[type]/[project]/versions.vue'))

const layout = useModerationReviewLayout()
const { projectV2: project, projectV3, versions, loadDependencies } =
	injectProjectPageContext()

onMounted(() => loadDependencies())

const selectedVersion = computed(() =>
	layout.selectedVersionId.value
		? versions.value?.find((v) => v.id === layout.selectedVersionId.value)
		: undefined,
)

function resolveVersionId(segment: string): string | null {
	const decoded = decodeURIComponent(segment)
	const match = versions.value?.find(
		(v) =>
			v.id === decoded ||
			(v as { displayUrlEnding?: string }).displayUrlEnding === decoded ||
			v.version_number === decoded,
	)
	return match?.id ?? null
}

// function onClickCapture(event: MouseEvent) {
// 	if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey) return
// 	const anchor = (event.target as HTMLElement | null)?.closest('a[href*="/version/"]')
// 	if (!(anchor instanceof HTMLAnchorElement)) return
// 	const seg = anchor.getAttribute('href')?.split('/version/')[1]?.split(/[?#]/)[0]
// 	if (!seg) return
// 	const id = resolveVersionId(seg)
// 	if (!id) return
// 	event.preventDefault()
// 	event.stopPropagation()
// 	layout.setSelectedVersion(id)
// }
</script>
