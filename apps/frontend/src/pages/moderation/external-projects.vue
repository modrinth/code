<template>
	<div>
		<form class="flex gap-2" @submit.prevent="executeSearch">
			<StyledInput
				v-model="query"
				:icon="SearchIcon"
				type="text"
				autocomplete="off"
				:placeholder="formatMessage(messages.searchPlaceholder)"
				clearable
				wrapper-class="flex-1 w-full"
			/>
			<ButtonStyled color="brand">
				<button type="submit">
					<SearchIcon aria-hidden="true" />
					{{ formatMessage(messages.searchByTitle) }}
				</button>
			</ButtonStyled>
			<ButtonStyled>
				<button type="button" @click="executeFlameIdLookup">
					<BinaryIcon aria-hidden="true" />
					{{ formatMessage(messages.lookupCurseForgeId) }}
				</button>
			</ButtonStyled>
			<ButtonStyled>
				<button type="button" @click="executeSha1Lookup">
					<HashIcon aria-hidden="true" />
					{{ formatMessage(messages.lookupSha1Hash) }}
				</button>
			</ButtonStyled>
		</form>
		<div>
			<template v-if="lastSearchKind !== 'none'">
				<template v-if="lookupNeedsInput">
					<EmptyState
						type="no-search-result"
						:heading="lookupEmptyHeading"
						:description="lookupEmptyDescription"
					/>
				</template>
				<template v-else>
					<div v-if="displayProjects.length > 0" class="mt-4 flex flex-col gap-3">
						<ExternalProjectLookupCard
							v-for="project in displayProjects"
							:key="project.id"
							:title="project.title"
							:state="project.status"
							:link="project.link"
							:notes="project.proof"
							:proof="null"
							:files="project.files"
						/>
					</div>
					<EmptyState v-else type="no-search-result" :heading="noResultsHeading" />
				</template>
			</template>
			<EmptyState
				v-else
				type="no-search-result"
				:heading="formatMessage(messages.emptyTitle)"
				:description="formatMessage(messages.emptyDescription)"
			/>
		</div>
	</div>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { BinaryIcon, HashIcon, SearchIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	defineMessages,
	EmptyState,
	type ExternalLicenseStatus,
	ExternalProjectLookupCard,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { injectModrinthClient } from '@modrinth/ui'

type ExternalProject = {
	id: number
	title: string | null
	status: ExternalLicenseStatus
	link: string | null
	exceptions: string | null
	proof: string | null
	flame_project_id: number | null
	files: {
		sha1: string
		name: string | null
	}[]
}

type SearchKind = 'none' | 'title' | 'flame_id' | 'sha1'

function mapProject(p: Labrinth.ExternalLicense.Internal.ExternalProject): ExternalProject {
	return {
		id: p.id,
		title: p.title,
		status: p.status as ExternalLicenseStatus,
		link: p.link,
		exceptions: p.exceptions,
		proof: p.proof,
		flame_project_id: p.flame_project_id,
		files: p.linked_files.map((f) => ({ sha1: f.sha1, name: f.name })),
	}
}

const client = injectModrinthClient()
const query = ref('')

const { formatMessage } = useVIntl()

const messages = defineMessages({
	searchPlaceholder: {
		id: 'moderation.external-projects.search-placeholder',
		defaultMessage: 'Search external projects...',
	},
	searchByTitle: {
		id: 'moderation.external-projects.search-by-title',
		defaultMessage: 'Search by title',
	},
	lookupCurseForgeId: {
		id: 'moderation.external-projects.lookup-curseforge-id',
		defaultMessage: 'Lookup CurseForge ID',
	},
	lookupSha1Hash: {
		id: 'moderation.external-projects.lookup-sha1-hash',
		defaultMessage: 'Lookup SHA-1',
	},
	emptyTitle: {
		id: 'moderation.external-projects.empty-title',
		defaultMessage: 'Enter a search term to get started',
	},
	emptyDescription: {
		id: 'moderation.external-projects.empty-description',
		defaultMessage: 'Type at least 3 characters of a project\u2019s title to begin browsing.',
	},
	noResultsByTitle: {
		id: 'moderation.external-projects.no-results-by-title',
		defaultMessage: 'No projects matched that title search',
	},
	noResultsByFlameId: {
		id: 'moderation.external-projects.no-results-by-flame-id',
		defaultMessage: 'No external project has that CurseForge ID',
	},
	noResultsBySha1: {
		id: 'moderation.external-projects.no-results-by-sha1',
		defaultMessage: 'No external file has that SHA-1 hash',
	},
	emptyFlameIdTitle: {
		id: 'moderation.external-projects.empty-flame-id-title',
		defaultMessage: 'Enter a CurseForge project ID',
	},
	emptyFlameIdDescription: {
		id: 'moderation.external-projects.empty-flame-id-description',
		defaultMessage: 'Type the numeric project ID, then use lookup for an exact match.',
	},
	emptySha1Title: {
		id: 'moderation.external-projects.empty-sha1-title',
		defaultMessage: 'Enter a SHA-1 hash',
	},
	emptySha1Description: {
		id: 'moderation.external-projects.empty-sha1-description',
		defaultMessage: 'Paste the full 40-character hex hash, then use lookup for an exact match.',
	},
	pageTitle: {
		id: 'moderation.external-projects.page-title',
		defaultMessage: 'External projects - Modrinth',
	},
})

useHead({ title: () => formatMessage(messages.pageTitle) })

const lastSearchKind = ref<SearchKind>('none')
const activeQuery = ref('')
const externalProjects = ref<ExternalProject[]>([])
const isLoading = ref(false)
const hasSearched = ref(false)

const displayProjects = computed(() => {
	if (isLoading.value) {
		return []
	}
	return externalProjects.value
})

const lookupNeedsInput = computed(() => {
	const q = activeQuery.value
	const kind = lastSearchKind.value

	if (kind === 'title') {
		return q.length < 3
	}
	if (kind === 'flame_id' || kind === 'sha1') {
		return q.trim().length === 0
	}
	return false
})

const lookupEmptyHeading = computed(() => {
	const kind = lastSearchKind.value
	if (kind === 'title') {
		return formatMessage(messages.emptyTitle)
	}
	if (kind === 'flame_id') {
		return formatMessage(messages.emptyFlameIdTitle)
	}
	if (kind === 'sha1') {
		return formatMessage(messages.emptySha1Title)
	}
	return ''
})

const lookupEmptyDescription = computed(() => {
	const kind = lastSearchKind.value
	if (kind === 'title') {
		return formatMessage(messages.emptyDescription)
	}
	if (kind === 'flame_id') {
		return formatMessage(messages.emptyFlameIdDescription)
	}
	if (kind === 'sha1') {
		return formatMessage(messages.emptySha1Description)
	}
	return ''
})

const noResultsHeading = computed(() => {
	const kind = lastSearchKind.value
	if (kind === 'title') {
		return formatMessage(messages.noResultsByTitle)
	}
	if (kind === 'flame_id') {
		return formatMessage(messages.noResultsByFlameId)
	}
	if (kind === 'sha1') {
		return formatMessage(messages.noResultsBySha1)
	}
	return formatMessage(messages.noResultsByTitle)
})

async function executeSearch() {
	lastSearchKind.value = 'title'
	activeQuery.value = query.value

	if (query.value.length < 3) {
		externalProjects.value = []
		return
	}

	isLoading.value = true
	try {
		const results = await client.labrinth.external_license_internal.search({
			title: query.value,
		})
		externalProjects.value = results.map(mapProject)
	} catch {
		externalProjects.value = []
	} finally {
		isLoading.value = false
		hasSearched.value = true
	}
}

async function executeFlameIdLookup() {
	lastSearchKind.value = 'flame_id'
	activeQuery.value = query.value

	const flameId = parseInt(query.value.trim(), 10)
	if (isNaN(flameId)) {
		externalProjects.value = []
		return
	}

	isLoading.value = true
	try {
		const results = await client.labrinth.external_license_internal.search({
			flame_id: flameId,
		})
		externalProjects.value = results.map(mapProject)
	} catch {
		externalProjects.value = []
	} finally {
		isLoading.value = false
		hasSearched.value = true
	}
}

async function executeSha1Lookup() {
	lastSearchKind.value = 'sha1'
	activeQuery.value = query.value

	const sha1 = query.value.trim()
	if (!sha1) {
		externalProjects.value = []
		return
	}

	isLoading.value = true
	try {
		const result = await client.labrinth.external_license_internal.getBySha1(sha1)
		externalProjects.value = [mapProject(result)]
	} catch {
		externalProjects.value = []
	} finally {
		isLoading.value = false
		hasSearched.value = true
	}
}
</script>
