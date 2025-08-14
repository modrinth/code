<template>
	<div class="flex flex-col gap-3">
		<div class="flex flex-col justify-between gap-3 lg:flex-row">
			<div class="iconified-input flex-1 lg:max-w-md">
				<SearchIcon aria-hidden="true" class="text-lg" />
				<input
					v-model="query"
					class="h-[40px]"
					autocomplete="off"
					spellcheck="false"
					type="text"
					:placeholder="formatMessage(messages.searchPlaceholder)"
					@input="updateSearchResults()"
				/>
				<Button v-if="query" class="r-btn" @click="() => (query = '')">
					<XIcon />
				</Button>
			</div>

			<div v-if="totalPages > 1" class="hidden flex-1 justify-center lg:flex">
				<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
			</div>

			<div class="flex flex-col justify-end gap-2 sm:flex-row lg:flex-shrink-0">
				<DropdownSelect
					v-slot="{ selected }"
					v-model="currentFilterType"
					class="!w-full flex-grow sm:!w-[280px] sm:flex-grow-0 lg:!w-[280px]"
					:name="formatMessage(messages.filterBy)"
					:options="filterTypes as unknown[]"
					@change="updateSearchResults()"
				>
					<span class="flex flex-row gap-2 align-middle font-semibold text-secondary">
						<FilterIcon class="size-4 flex-shrink-0" />
						<span class="truncate">{{ selected }} ({{ filteredReports.length }})</span>
					</span>
				</DropdownSelect>

				<DropdownSelect
					v-slot="{ selected }"
					v-model="currentSortType"
					class="!w-full flex-grow sm:!w-[150px] sm:flex-grow-0 lg:!w-[150px]"
					:name="formatMessage(messages.sortBy)"
					:options="sortTypes as unknown[]"
					@change="updateSearchResults()"
				>
					<span class="flex flex-row gap-2 align-middle font-semibold text-secondary">
						<SortAscIcon v-if="selected === 'Oldest'" class="size-4 flex-shrink-0" />
						<SortDescIcon v-else class="size-4 flex-shrink-0" />
						<span class="truncate">{{ selected }}</span>
					</span>
				</DropdownSelect>
			</div>
		</div>

		<div v-if="totalPages > 1" class="flex justify-center lg:hidden">
			<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
		</div>

		<div class="mt-4 flex flex-col gap-2">
			<DelphiReportCard
				v-for="report in paginatedReports"
				:key="report.version.id"
				:report="report"
			/>
			<div
				v-if="!paginatedReports || paginatedReports.length === 0"
				class="universal-card h-24 animate-pulse"
			></div>
		</div>

		<div v-if="totalPages > 1" class="mt-4 flex justify-center">
			<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
		</div>
	</div>
</template>

<script setup lang="ts">
import { FilterIcon, SearchIcon, SortAscIcon, SortDescIcon, XIcon } from '@modrinth/assets'
import type { ExtendedDelphiReport, OwnershipTarget } from '@modrinth/moderation'
import { Button, DropdownSelect, Pagination } from '@modrinth/ui'
import type { DelphiReport, Organization, Project, TeamMember, Version } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { useLocalStorage } from '@vueuse/core'
import Fuse from 'fuse.js'

import DelphiReportCard from '~/components/ui/moderation/ModerationDelphiReportCard.vue'
import { asEncodedJsonArray, fetchSegmented } from '~/utils/fetch-helpers.ts'

const { formatMessage } = useVIntl()
const route = useRoute()
const router = useRouter()

const messages = defineMessages({
	searchPlaceholder: {
		id: 'moderation.technical.search.placeholder',
		defaultMessage: 'Search tech reviews...',
	},
	filterBy: {
		id: 'moderation.filter.by',
		defaultMessage: 'Filter by',
	},
	sortBy: {
		id: 'moderation.sort.by',
		defaultMessage: 'Sort by',
	},
})

async function getProjectQuicklyForMock(projectId: string): Promise<Project> {
	return (await useBaseFetch(`project/${projectId}`)) as Project
}

async function getVersionQuicklyForMock(versionId: string): Promise<Version> {
	return (await useBaseFetch(`version/${versionId}`)) as Version
}

const mockDelphiReports: DelphiReport[] = [
	{
		project: await getProjectQuicklyForMock('7MoE34WK'),
		version: await getVersionQuicklyForMock('cTkKLWgA'),
		trace_type: 'url_usage',
		file_path: 'me/decce/gnetum/ASMEventHandlerHelper.java',
		priority_score: 29,
		status: 'pending',
		detected_at: '2025-04-01T12:00:00Z',
	} as DelphiReport,
	{
		project: await getProjectQuicklyForMock('7MoE34WK'),
		version: await getVersionQuicklyForMock('cTkKLWgA'),
		trace_type: 'url_usage',
		file_path: 'me/decce/gnetum/SomeOtherFile.java',
		priority_score: 48,
		status: 'rejected',
		detected_at: '2025-03-02T12:00:00Z',
	} as DelphiReport,
	{
		project: await getProjectQuicklyForMock('7MoE34WK'),
		version: await getVersionQuicklyForMock('cTkKLWgA'),
		trace_type: 'url_usage',
		file_path: 'me/decce/gnetum/YetAnotherFile.java',
		priority_score: 15,
		status: 'approved',
		detected_at: '2025-02-03T12:00:00Z',
	} as DelphiReport,
]

const { data: allReports } = await useAsyncData('moderation-tech-reviews', async () => {
	// TODO: replace with actual API call
	const delphiReports = mockDelphiReports

	if (delphiReports.length === 0) {
		return []
	}

	const teamIds = [...new Set(delphiReports.map((report) => report.project.team).filter(Boolean))]
	const orgIds = [
		...new Set(delphiReports.map((report) => report.project.organization).filter(Boolean)),
	]

	const [teamsData, orgsData]: [TeamMember[][], Organization[]] = await Promise.all([
		teamIds.length > 0
			? fetchSegmented(teamIds, (ids) => `teams?ids=${asEncodedJsonArray(ids)}`)
			: Promise.resolve([]),
		orgIds.length > 0
			? fetchSegmented(orgIds, (ids) => `organizations?ids=${asEncodedJsonArray(ids)}`, {
					apiVersion: 3,
				})
			: Promise.resolve([]),
	])

	const orgTeamIds = orgsData.map((org) => org.team_id).filter(Boolean)
	const orgTeamsData: TeamMember[][] =
		orgTeamIds.length > 0
			? await fetchSegmented(orgTeamIds, (ids) => `teams?ids=${asEncodedJsonArray(ids)}`)
			: []

	const teamMap = new Map<string, TeamMember[]>()
	const orgMap = new Map<string, Organization>()

	teamsData.forEach((team) => {
		let teamId = null
		for (const member of team) {
			teamId = member.team_id
			if (!teamMap.has(teamId)) {
				teamMap.set(teamId, team)
				break
			}
		}
	})

	orgTeamsData.forEach((team) => {
		let teamId = null
		for (const member of team) {
			teamId = member.team_id
			if (!teamMap.has(teamId)) {
				teamMap.set(teamId, team)
				break
			}
		}
	})

	orgsData.forEach((org: Organization) => {
		orgMap.set(org.id, org)
	})

	const extendedReports: ExtendedDelphiReport[] = delphiReports.map((report) => {
		let target: OwnershipTarget | undefined
		const project = report.project

		if (project) {
			let owner: TeamMember | null = null
			let org: Organization | null = null

			if (project.team) {
				const teamMembers = teamMap.get(project.team)
				if (teamMembers) {
					owner = teamMembers.find((member) => member.role === 'Owner') || null
				}
			}

			if (project.organization) {
				org = orgMap.get(project.organization) || null
			}

			if (org) {
				target = {
					name: org.name,
					avatar_url: org.icon_url,
					type: 'organization',
					slug: org.slug,
				}
			} else if (owner) {
				target = {
					name: owner.user.username,
					avatar_url: owner.user.avatar_url,
					type: 'user',
					slug: owner.user.username,
				}
			}
		}

		return {
			...report,
			target,
		}
	})

	extendedReports.sort((a, b) => b.priority_score - a.priority_score)

	return extendedReports
})

const query = ref(route.query.q?.toString() || '')
watch(
	query,
	(newQuery) => {
		const currentQuery = { ...route.query }
		if (newQuery) {
			currentQuery.q = newQuery
		} else {
			delete currentQuery.q
		}

		router.replace({
			path: route.path,
			query: currentQuery,
		})
	},
	{ immediate: false },
)

watch(
	() => route.query.q,
	(newQueryParam) => {
		const newValue = newQueryParam?.toString() || ''
		if (query.value !== newValue) {
			query.value = newValue
		}
	},
)

const currentFilterType = useLocalStorage('moderation-tech-reviews-filter-type', () => 'Pending')
const filterTypes: readonly string[] = readonly(['All', 'Pending', 'Approved', 'Rejected'])

const currentSortType = useLocalStorage('moderation-tech-reviews-sort-type', () => 'Priority')
const sortTypes: readonly string[] = readonly(['Priority', 'Oldest', 'Newest'])

const currentPage = ref(1)
const itemsPerPage = 15
const totalPages = computed(() => Math.ceil((filteredReports.value?.length || 0) / itemsPerPage))

const fuse = computed(() => {
	if (!allReports.value || allReports.value.length === 0) return null
	return new Fuse(allReports.value, {
		keys: [
			{
				name: 'version.id',
				weight: 3,
			},
			{
				name: 'version.version_number',
				weight: 3,
			},
			{
				name: 'project.title',
				weight: 3,
			},
			{
				name: 'project.slug',
				weight: 3,
			},
			{
				name: 'version.files.filename',
				weight: 2,
			},
			{
				name: 'trace_type',
				weight: 2,
			},
			{
				name: 'content',
				weight: 0.5,
			},
			'file_path',
			'project.id',
			'target.name',
			'target.slug',
		],
		includeScore: true,
		threshold: 0.4,
	})
})

const filteredReports = computed(() => {
	if (!allReports.value) return []

	let filtered

	if (query.value && fuse.value) {
		const results = fuse.value.search(query.value)
		filtered = results.map((result) => result.item)
	} else {
		filtered = [...allReports.value]
	}

	if (currentFilterType.value === 'Pending') {
		filtered = filtered.filter((report) => report.status === 'pending')
	} else if (currentFilterType.value === 'Approved') {
		filtered = filtered.filter((report) => report.status === 'approved')
	} else if (currentFilterType.value === 'Rejected') {
		filtered = filtered.filter((report) => report.status === 'rejected')
	}

	if (currentSortType.value === 'Priority') {
		filtered.sort((a, b) => b.priority_score - a.priority_score)
	} else if (currentSortType.value === 'Oldest') {
		filtered.sort((a, b) => {
			const dateA = new Date(a.detected_at).getTime()
			const dateB = new Date(b.detected_at).getTime()
			return dateA - dateB
		})
	} else {
		filtered.sort((a, b) => {
			const dateA = new Date(a.detected_at).getTime()
			const dateB = new Date(b.detected_at).getTime()
			return dateB - dateA
		})
	}

	return filtered
})

const paginatedReports = computed(() => {
	if (!filteredReports.value) return []
	const start = (currentPage.value - 1) * itemsPerPage
	const end = start + itemsPerPage
	return filteredReports.value.slice(start, end)
})

function updateSearchResults() {
	currentPage.value = 1
}

function goToPage(page: number) {
	currentPage.value = page
}
</script>
