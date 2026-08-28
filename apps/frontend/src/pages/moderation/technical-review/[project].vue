<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { LoaderCircleIcon } from '@modrinth/assets'
import { BackToParentLink, injectModrinthClient } from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'

import MaliciousSummaryModal, {
	type UnsafeFile,
} from '~/components/ui/moderation/MaliciousSummaryModal.vue'
import ModerationTechRevCard from '~/components/ui/moderation/ModerationTechRevCard.vue'
import { flattenFileReports } from '~/components/ui/moderation/tech-review/helpers'
import { useTechReviewSources } from '~/components/ui/moderation/tech-review/use-tech-review-sources'

const client = injectModrinthClient()
const queryClient = useQueryClient()
const route = useRoute()
const keybinds = useModerationKeybinds()

const projectId = String(useRouteId('project'))

useHead({ title: () => `Tech review - ${projectId} - Modrinth` })

const {
	data: projectReportData,
	isLoading: isLoadingReport,
	isError: isReportError,
} = useQuery({
	queryKey: ['tech-review-project-report', projectId],
	queryFn: async () => {
		return await client.labrinth.tech_review_internal.getProjectReport(projectId)
	},
	retry: false,
})

const {
	data: projectData,
	isLoading: isLoadingProject,
	isError: isProjectError,
} = useQuery({
	queryKey: ['project', projectId],
	queryFn: async () => {
		return await client.labrinth.projects_v3.get(projectId)
	},
	retry: false,
})

const { data: organizationData, isLoading: isLoadingOrg } = useQuery({
	queryKey: ['project-organization', projectId],
	queryFn: async () => {
		return await client.labrinth.projects_v3.getOrganization(projectId)
	},
})

const { data: membersData, isLoading: isLoadingMembers } = useQuery({
	queryKey: ['project-members', projectId],
	queryFn: async () => {
		return await client.labrinth.projects_v3.getMembers(projectId)
	},
	enabled: computed(() => !organizationData.value && !isLoadingOrg.value),
})

const isLoading = computed(
	() =>
		isLoadingReport.value ||
		isLoadingProject.value ||
		isLoadingOrg.value ||
		(isLoadingMembers.value && !organizationData.value),
)

const hasError = computed(() => isReportError.value || isProjectError.value)

const ownership = computed<Labrinth.TechReview.Internal.Ownership | null>(() => {
	if (organizationData.value) {
		return {
			kind: 'organization',
			id: organizationData.value.id,
			name: organizationData.value.name,
			icon_url: organizationData.value.icon_url ?? undefined,
		}
	}

	if (membersData.value) {
		const owner = membersData.value.find((m) => m.is_owner)
		if (owner) {
			return {
				kind: 'user',
				id: owner.user.id,
				name: owner.user.username,
				icon_url: owner.user.avatar_url ?? undefined,
			}
		}
	}

	return null
})

const reviewItem = computed(() => {
	if (!projectReportData.value || !projectData.value || !ownership.value) return null

	const { project_report, thread } = projectReportData.value

	const reports = project_report ? flattenFileReports(project_report.versions) : []

	return {
		project: projectData.value,
		project_owner: ownership.value,
		thread,
		reports,
	}
})

const { loadingIssues, decompiledSources, handleLoadIssueSources } = useTechReviewSources(
	() => reviewItem.value?.reports.flatMap((report) => report.issues) ?? [],
)

const focusedDetailId = computed(() => route.query.detail?.toString() ?? null)

async function handleMarkComplete(projectId: string) {
	await Promise.all([
		queryClient.invalidateQueries({ queryKey: ['tech-reviews'] }),
		queryClient.invalidateQueries({ queryKey: ['tech-review-project-report', projectId] }),
		queryClient.invalidateQueries({ queryKey: ['project', projectId] }),
		queryClient.invalidateQueries({ queryKey: ['project', 'v2', projectId] }),
		queryClient.invalidateQueries({ queryKey: ['project', 'v3', projectId] }),
	])
}

const maliciousSummaryModalRef = ref<InstanceType<typeof MaliciousSummaryModal>>()
const currentUnsafeFiles = ref<UnsafeFile[]>([])

function handleShowMaliciousSummary(unsafeFiles: UnsafeFile[]) {
	currentUnsafeFiles.value = unsafeFiles
	maliciousSummaryModalRef.value?.show()
}

function refetch() {
	queryClient.invalidateQueries({ queryKey: ['tech-review-project-report', projectId] })
}

function handleKeybinds(event: KeyboardEvent) {
	keybinds.value.handle(event, {
		scope: 'tech-review',
		actions: {
			goToTop: () => {
				window.scrollTo({
					top: 0,
					behavior: 'smooth',
				})
			},
			goToBottom: () => {
				window.scrollTo({
					top: document.body.scrollHeight,
					behavior: 'smooth',
				})
			},
		},
	})
}

onMounted(() => {
	window.addEventListener('keydown', handleKeybinds)
})

onUnmounted(() => {
	window.removeEventListener('keydown', handleKeybinds)
})
</script>

<template>
	<div class="flex flex-col">
		<BackToParentLink :to="'/moderation/technical-review'"> Back to queue </BackToParentLink>

		<div v-if="isLoading" class="flex flex-col gap-4">
			<div class="universal-card flex h-48 items-center justify-center">
				<LoaderCircleIcon class="size-8 animate-spin text-secondary" />
			</div>
		</div>
		<div
			v-else-if="hasError"
			class="universal-card flex h-24 items-center justify-center text-secondary"
		>
			Project not found in the tech review queue.
		</div>
		<div
			v-else-if="!reviewItem"
			class="universal-card flex h-24 items-center justify-center text-secondary"
		>
			No review data available for this project.
		</div>
		<ModerationTechRevCard
			v-else
			:item="reviewItem"
			:focused-detail-id="focusedDetailId"
			:loading-issues="loadingIssues"
			:decompiled-sources="decompiledSources"
			:collapsed="false"
			disable-collapsing
			@refetch="refetch"
			@load-issue-sources="handleLoadIssueSources"
			@mark-complete="handleMarkComplete"
			@show-malicious-summary="handleShowMaliciousSummary"
		/>
		<MaliciousSummaryModal ref="maliciousSummaryModalRef" :unsafe-files="currentUnsafeFiles" />
	</div>
</template>
