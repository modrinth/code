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
				<div class="flex flex-wrap items-start justify-between gap-3">
					<div class="min-w-0">
						<div class="flex min-w-0 items-center gap-2">
							<HashIcon class="shrink-0 text-secondary" aria-hidden="true" />
							<h2 class="m-0 min-w-0 text-lg font-semibold text-contrast">
								Trace
								<span class="break-all font-mono text-base">{{ trace.detail_key }}</span>
							</h2>
						</div>
						<p class="m-0 mt-1 text-sm text-secondary">
							{{ formatTraceCount(trace.local_trace_count) }}
						</p>
					</div>
					<Badge :type="trace.verdict" />
				</div>

				<div v-if="getPreviewLocalTraces(trace).length > 0" class="flex flex-col gap-2">
					<div
						v-if="getVisibleLocalTraceTotal(trace) > getPreviewLocalTraces(trace).length"
						class="flex flex-wrap items-center justify-between gap-2"
					>
						<p class="m-0 text-sm text-secondary">
							Showing first {{ getPreviewLocalTraces(trace).length }} of
							{{ getVisibleLocalTraceTotal(trace) }} local traces
						</p>
						<ButtonStyled>
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
import { HashIcon, ListIcon, SearchIcon } from '@modrinth/assets'
import {
	Badge,
	ButtonStyled,
	EmptyState,
	injectModrinthClient,
	Pagination,
	StyledInput,
} from '@modrinth/ui'

import GlobalDetailLocalTraceCard from '~/components/ui/moderation/GlobalDetailLocalTraceCard.vue'

const client = injectModrinthClient()
const query = ref('')
const activeQuery = ref<string | null>(null)
const isLoading = ref(false)
const loadError = ref(false)
const currentPage = ref(1)
const itemsPerPage = 20
const localTracePreviewLimit = 10
const total = ref(0)
const traces = ref<Labrinth.TechReview.Internal.GlobalIssueDetail[]>([])

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

onMounted(loadTraces)
</script>
