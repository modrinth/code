<template>
	<div>
		<form class="flex flex-col gap-2 sm:flex-row" @submit.prevent="executeSearch">
			<StyledInput
				v-model="query"
				:icon="SearchIcon"
				type="text"
				autocomplete="off"
				placeholder="Search global trace keys..."
				clearable
				wrapper-class="flex-1 w-full"
			/>
			<ButtonStyled color="brand">
				<button type="submit" :disabled="isLoading">
					<SearchIcon aria-hidden="true" />
					Search
				</button>
			</ButtonStyled>
		</form>

		<div
			v-if="!isLoading && !loadError && total > 0"
			class="mt-4 flex flex-wrap items-center justify-between gap-3"
		>
			<p class="m-0 text-sm text-secondary">Showing {{ pageStart }}-{{ pageEnd }} of {{ total }}</p>
			<Pagination :page="currentPage" :count="pageCount" @switch-page="switchPage" />
		</div>

		<EmptyState
			v-if="isLoading"
			type="no-search-result"
			heading="Loading global detail traces..."
		/>
		<EmptyState
			v-else-if="loadError"
			type="no-search-result"
			heading="Failed to load global detail traces"
		/>
		<div v-else-if="traces.length > 0" class="mt-4 flex flex-col gap-3">
			<article
				v-for="trace in traces"
				:key="trace.detail_key"
				class="universal-card flex flex-col gap-3"
			>
				<div class="grid grid-cols-[minmax(0,1fr)_auto] items-start gap-3">
					<div class="min-w-0">
						<div class="flex min-w-0 flex-wrap items-center gap-2">
							<HashIcon class="shrink-0 text-secondary" aria-hidden="true" />
							<h2 class="m-0 min-w-0 text-lg font-semibold text-contrast">
								Trace
								<span class="break-all font-mono text-base">{{ trace.detail_key }}</span>
							</h2>
							<span
								v-if="getLatestLocalTrace(trace)"
								class="rounded-full border border-solid px-2.5 py-1 text-sm font-medium capitalize"
								:class="getSeverityBadgeColor(getLatestLocalTrace(trace)?.severity)"
							>
								{{ getLatestLocalTrace(trace)?.severity }}
							</span>
						</div>
						<div v-if="getLatestLocalTrace(trace)" class="mt-1 flex flex-wrap gap-x-3 text-sm">
							<p class="m-0 break-all text-secondary">
								<span class="font-semibold text-contrast">Issue</span>
								{{ getLatestLocalTrace(trace)?.issue_type }}
							</p>
							<p class="m-0 break-all text-secondary">
								<span class="font-semibold text-contrast">Path</span>
								{{ decodeTracePath(getLatestLocalTrace(trace)?.file_path ?? '') }}
							</p>
						</div>
					</div>
					<div class="flex shrink-0 flex-nowrap items-center gap-2">
						<Badge :type="trace.verdict" />
						<ButtonStyled color="red">
							<button
								:disabled="removingTraceKeys.has(trace.detail_key)"
								@click="removeGlobalTrace(trace)"
							>
								<TrashIcon aria-hidden="true" />
								Remove
							</button>
						</ButtonStyled>
					</div>
				</div>

				<div v-if="getPreviewLocalTraces(trace).length > 0" class="flex flex-col gap-2">
					<div class="flex flex-wrap items-center justify-between gap-2">
						<p class="m-0 text-sm text-secondary">
							Showing {{ getPreviewLocalTraces(trace).length }} of
							{{ formatTraceCount(getVisibleLocalTraceTotal(trace)) }}
						</p>
						<ButtonStyled
							v-if="getVisibleLocalTraceTotal(trace) > getPreviewLocalTraces(trace).length"
						>
							<NuxtLink :to="getGlobalTraceLink(trace)">
								<ListIcon aria-hidden="true" />
								View all
							</NuxtLink>
						</ButtonStyled>
					</div>
					<GlobalDetailLocalTraceCard
						v-for="localTrace in getPreviewLocalTraces(trace)"
						:key="localTrace.detail_id"
						:trace="localTrace"
					/>
				</div>
				<EmptyState
					v-else
					type="no-search-result"
					heading="No local traces currently match this key"
				/>
			</article>
		</div>
		<EmptyState v-else type="no-search-result" heading="No global detail traces found" />

		<div v-if="!isLoading && !loadError && total > 0" class="mt-4 flex justify-end">
			<Pagination :page="currentPage" :count="pageCount" @switch-page="switchPage" />
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { HashIcon, ListIcon, SearchIcon, TrashIcon } from '@modrinth/assets'
import {
	Badge,
	ButtonStyled,
	EmptyState,
	injectModrinthClient,
	injectNotificationManager,
	Pagination,
	StyledInput,
} from '@modrinth/ui'

import GlobalDetailLocalTraceCard from '~/components/ui/moderation/GlobalDetailLocalTraceCard.vue'

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const query = ref('')
const activeQuery = ref<string | null>(null)
const isLoading = ref(false)
const loadError = ref(false)
const currentPage = ref(1)
const itemsPerPage = 20
const localTracePreviewLimit = 3
const total = ref(0)
const traces = ref<Labrinth.TechReview.Internal.GlobalIssueDetail[]>([])
const removingTraceKeys = reactive<Set<string>>(new Set())

const pageCount = computed(() => Math.max(Math.ceil(total.value / itemsPerPage), 1))
const pageStart = computed(() =>
	total.value === 0 ? 0 : (currentPage.value - 1) * itemsPerPage + 1,
)
const pageEnd = computed(() => Math.min(currentPage.value * itemsPerPage, total.value))

function formatTraceCount(count: number) {
	return `${count} local ${count === 1 ? 'trace' : 'traces'}`
}

function getPreviewLocalTraces(trace: Labrinth.TechReview.Internal.GlobalIssueDetail) {
	return trace.local_traces.slice(0, localTracePreviewLimit)
}

function getLatestLocalTrace(trace: Labrinth.TechReview.Internal.GlobalIssueDetail) {
	return trace.local_traces.at(-1)
}

function decodeTracePath(path: string): string {
	try {
		return decodeURIComponent(path)
	} catch {
		return path
	}
}

function getSeverityBadgeColor(
	severity: Labrinth.TechReview.Internal.DelphiSeverity | undefined,
): string {
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

function getVisibleLocalTraceTotal(trace: Labrinth.TechReview.Internal.GlobalIssueDetail) {
	return Math.max(trace.local_trace_count, trace.local_traces.length)
}

function getGlobalTraceLink(trace: Labrinth.TechReview.Internal.GlobalIssueDetail) {
	return `/moderation/global-traces/${encodeURIComponent(trace.detail_key)}`
}

async function loadTraces() {
	isLoading.value = true
	loadError.value = false

	try {
		const response = await client.labrinth.tech_review_internal.searchGlobalIssueDetails({
			query: activeQuery.value,
			limit: itemsPerPage,
			page: currentPage.value - 1,
		})

		traces.value = response.traces
		total.value = response.total
	} catch (error) {
		console.error('Failed to load global detail traces', error)
		traces.value = []
		total.value = 0
		loadError.value = true
	} finally {
		isLoading.value = false
	}
}

async function executeSearch() {
	activeQuery.value = query.value.trim() || null
	currentPage.value = 1
	await loadTraces()
}

async function switchPage(page: number) {
	currentPage.value = page
	await loadTraces()
}

async function removeGlobalTrace(trace: Labrinth.TechReview.Internal.GlobalIssueDetail) {
	if (removingTraceKeys.has(trace.detail_key)) return

	removingTraceKeys.add(trace.detail_key)
	try {
		await client.labrinth.tech_review_internal.updateGlobalIssueDetails([
			{ detail_key: trace.detail_key, verdict: 'pending' },
		])

		addNotification({
			type: 'success',
			title: 'Global trace removed',
			text: 'The global verdict for this trace key has been removed.',
		})

		if (traces.value.length === 1 && currentPage.value > 1) {
			currentPage.value--
		}

		await loadTraces()
	} catch (error) {
		console.error('Failed to remove global trace', error)
		addNotification({
			type: 'error',
			title: 'Failed to remove global trace',
			text: 'An error occurred while removing the global trace verdict.',
		})
	} finally {
		removingTraceKeys.delete(trace.detail_key)
	}
}

onMounted(loadTraces)
</script>
