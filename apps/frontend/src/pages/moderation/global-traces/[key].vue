<template>
	<div class="flex flex-col gap-4">
		<div>
			<ButtonStyled>
				<NuxtLink to="/moderation/global-traces">
					<ArrowLeftIcon aria-hidden="true" />
					Back to global traces
				</NuxtLink>
			</ButtonStyled>
		</div>

		<EmptyState
			v-if="isLoading && !trace"
			type="no-search-result"
			heading="Loading global detail trace..."
		/>
		<EmptyState
			v-else-if="loadError"
			type="no-search-result"
			heading="Failed to load global detail trace"
		/>
		<article v-else-if="trace" class="universal-card flex flex-col gap-3">
			<div class="grid grid-cols-[minmax(0,1fr)_auto] items-start gap-3">
				<div class="min-w-0">
					<div class="flex min-w-0 flex-wrap items-center gap-2">
						<HashIcon class="shrink-0 text-secondary" aria-hidden="true" />
						<h2 class="m-0 min-w-0 text-lg font-semibold text-contrast">
							Trace
							<span class="break-all font-mono text-base">{{ trace.detail_key }}</span>
						</h2>
						<span
							v-if="latestLocalTrace"
							class="rounded-full border border-solid px-2.5 py-1 text-sm font-medium capitalize"
							:class="getSeverityBadgeColor(latestLocalTrace.severity)"
						>
							{{ latestLocalTrace.severity }}
						</span>
					</div>
					<div v-if="latestLocalTrace" class="mt-1 flex flex-wrap gap-x-3 text-sm">
						<p class="m-0 break-all text-secondary">
							<span class="font-semibold text-contrast">Issue</span>
							{{ latestLocalTrace.issue_type }}
						</p>
						<p class="m-0 break-all text-secondary">
							<span class="font-semibold text-contrast">Path</span>
							{{ decodeTracePath(latestLocalTrace.file_path) }}
						</p>
					</div>
				</div>
				<div class="flex shrink-0 flex-nowrap items-center gap-2">
					<Badge :type="trace.verdict" />
					<ButtonStyled color="red">
						<button :disabled="isRemoving" @click="removeGlobalTrace">
							<TrashIcon aria-hidden="true" />
							Remove
						</button>
					</ButtonStyled>
				</div>
			</div>

			<div
				class="flex flex-wrap items-center justify-between gap-3 border-0 border-t border-solid border-divider pt-3"
			>
				<p class="m-0 text-sm text-secondary">
					{{ pageStart }}-{{ pageEnd }} of {{ trace.local_trace_count }} local traces
					<span v-if="isLoading"> · Loading page...</span>
				</p>
				<Pagination
					v-if="trace.local_trace_count > localTracePageSize"
					:page="currentPage"
					:count="pageCount"
					@switch-page="switchPage"
				/>
			</div>

			<div v-if="trace.local_traces.length > 0" class="flex flex-col gap-2">
				<GlobalDetailLocalTraceCard
					v-for="localTrace in trace.local_traces"
					:key="localTrace.detail_id"
					:trace="localTrace"
				/>
			</div>
			<EmptyState v-else type="no-search-result" heading="No local traces match this key" />

			<div v-if="trace.local_trace_count > localTracePageSize" class="mt-1 flex justify-end">
				<Pagination :page="currentPage" :count="pageCount" @switch-page="switchPage" />
			</div>
		</article>
		<EmptyState v-else type="no-search-result" heading="Global detail trace not found" />
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ArrowLeftIcon, HashIcon, TrashIcon } from '@modrinth/assets'
import {
	Badge,
	ButtonStyled,
	EmptyState,
	injectModrinthClient,
	injectNotificationManager,
	Pagination,
} from '@modrinth/ui'

import GlobalDetailLocalTraceCard from '~/components/ui/moderation/GlobalDetailLocalTraceCard.vue'

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const route = useRoute()
const router = useRouter()

const detailKey = computed(() => {
	const key = route.params.key
	return Array.isArray(key) ? key.join('/') : String(key)
})

useHead({ title: () => `Global trace - ${detailKey.value} - Modrinth` })

const localTracePageSize = 20
const isLoading = ref(false)
const isRemoving = ref(false)
const loadError = ref(false)
const currentPage = ref(1)
const pageStartCursors = ref<(string | null)[]>([null])
const trace = ref<Labrinth.TechReview.Internal.GlobalIssueDetail | null>(null)

const pageCount = computed(() =>
	Math.max(Math.ceil((trace.value?.local_trace_count ?? 0) / localTracePageSize), 1),
)
const pageStart = computed(() =>
	trace.value && trace.value.local_trace_count > 0
		? (currentPage.value - 1) * localTracePageSize + 1
		: 0,
)
const pageEnd = computed(() =>
	Math.min(currentPage.value * localTracePageSize, trace.value?.local_trace_count ?? 0),
)
const latestLocalTrace = computed(() => trace.value?.local_traces.at(-1))

function decodeTracePath(path: string): string {
	try {
		return decodeURIComponent(path)
	} catch {
		return path
	}
}

function getSeverityBadgeColor(severity: Labrinth.TechReview.Internal.DelphiSeverity): string {
	switch (severity) {
		case 'severe':
			return 'border-red/60 bg-highlight-red text-red'
		case 'high':
			return 'border-orange/60 bg-highlight-orange text-orange'
		case 'medium':
			return 'border-green/60 bg-highlight-green text-green'
		case 'low':
		default:
			return 'border-blue/60 bg-highlight-blue text-blue'
	}
}

async function fetchTracePage(afterDetailId: string | null) {
	return await client.labrinth.tech_review_internal.getGlobalIssueDetail({
		detail_key: detailKey.value,
		limit: localTracePageSize,
		after_detail_id: afterDetailId,
	})
}

async function loadPage(page: number) {
	if (page < 1 || isLoading.value) return

	isLoading.value = true
	loadError.value = false

	try {
		while (pageStartCursors.value.length < page) {
			const cursor = pageStartCursors.value[pageStartCursors.value.length - 1]
			const response = await fetchTracePage(cursor)

			if (!response.next_after_detail_id) {
				trace.value = response.trace
				currentPage.value = pageStartCursors.value.length
				return
			}

			pageStartCursors.value.push(response.next_after_detail_id)
		}

		const response = await fetchTracePage(pageStartCursors.value[page - 1])
		trace.value = response.trace
		currentPage.value = page
	} catch (error) {
		console.error('Failed to load global detail trace', error)
		loadError.value = true
	} finally {
		isLoading.value = false
	}
}

async function switchPage(page: number) {
	await loadPage(page)
}

async function removeGlobalTrace() {
	if (isRemoving.value) return

	isRemoving.value = true
	try {
		await client.labrinth.tech_review_internal.updateGlobalIssueDetails([
			{ detail_key: detailKey.value, verdict: 'pending' },
		])

		addNotification({
			type: 'success',
			title: 'Global trace removed',
			text: 'The global verdict for this trace key has been removed.',
		})

		await router.push('/moderation/global-traces')
	} catch (error) {
		console.error('Failed to remove global detail trace', error)
		addNotification({
			type: 'error',
			title: 'Failed to remove global trace',
			text: 'An error occurred while removing the global trace verdict.',
		})
	} finally {
		isRemoving.value = false
	}
}

watch(
	detailKey,
	() => {
		currentPage.value = 1
		pageStartCursors.value = [null]
		trace.value = null
		loadPage(1)
	},
	{ immediate: true },
)
</script>
