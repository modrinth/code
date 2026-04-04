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
		defaultMessage: 'Type at least 3 characters of a project’s title to begin browsing.',
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

type ExternalProject = {
	id: string
	title: string | null
	status: ExternalLicenseStatus
	link: string | null
	exceptions: string | null
	proof: string | null
	flame_project_id: string | null
	files: {
		sha1: string
		name: string | null
	}[]
}

type SearchKind = 'none' | 'title' | 'flame_id' | 'sha1'

const lastSearchKind = ref<SearchKind>('none')
const activeQuery = ref('')

const externalProjects = ref<ExternalProject[]>([
	{
		id: 'iV1m7lMU',
		title: 'Create: Extended Flywheels Fabric',
		status: 'yes',
		link: 'https://www.curseforge.com/minecraft/mc-mods/create-extended-flywheels-fabric',
		proof: 'See CurseForge page/license for permission',
		exceptions: null,
		flame_project_id: '750818',
		files: [
			{
				sha1: 'd102ff70164359fb0906b32dbb067f4149e3239a',
				name: 'extendedflywheels-1.19.2-0.5.0.g-1.2.5.1-fabric.jar',
			},
			{
				sha1: '584de498addd512888b59d85f7278c2cbb9096c5',
				name: 'extendedflywheels-1.18.2-0.5.0.g-1.2.5.1-fabric.jar',
			},
			{
				sha1: 'fc887ab8f6dc40558dafb9ec35c4b08fb683e8f7',
				name: 'extendedflywheels-1.19.2-0.5.0.g-1.2.5-fabric.jar',
			},
			{
				sha1: 'b77e52f5d47820322d296427904553dee6dabbf6',
				name: 'extendedflywheels-1.18.2-0.5.0.g-1.2.5-fabric.jar',
			},
		],
	},
])

const displayProjects = computed(() => {
	const q = activeQuery.value
	const kind = lastSearchKind.value
	const projects = externalProjects.value

	if (kind === 'none') {
		return []
	}

	if (kind === 'title') {
		if (q.length < 3) {
			return []
		}
		const lower = q.toLowerCase()
		return projects.filter((p) => p.title?.toLowerCase().includes(lower) ?? false)
	}

	if (kind === 'flame_id') {
		const needle = q.trim()
		if (!needle) {
			return []
		}
		return projects.filter((p) => p.flame_project_id === needle)
	}

	if (kind === 'sha1') {
		const needle = q.trim().toLowerCase()
		if (!needle) {
			return []
		}
		return projects.filter((p) => p.files.some((f) => f.sha1.toLowerCase() === needle))
	}

	return []
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

function executeSearch() {
	lastSearchKind.value = 'title'
	activeQuery.value = query.value
}

function executeFlameIdLookup() {
	lastSearchKind.value = 'flame_id'
	activeQuery.value = query.value
}

function executeSha1Lookup() {
	lastSearchKind.value = 'sha1'
	activeQuery.value = query.value
}
</script>
