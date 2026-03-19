<template>
	<div class="universal-card">
		<CollectionCreateModal ref="modal_creation" />
		<h2 class="text-2xl">{{ formatMessage(commonMessages.collectionsLabel) }}</h2>
		<div class="mb-3 flex flex-col gap-3">
			<label for="search-input" hidden>{{ formatMessage(messages.searchInputLabel) }}</label>
			<StyledInput
				id="search-input"
				v-model="filterQuery"
				:icon="SearchIcon"
				type="text"
				clearable
				:placeholder="formatMessage(messages.searchCollectionsPlaceholder)"
				wrapper-class="w-full"
				input-class="!h-12"
			/>

			<div class="flex flex-wrap items-center gap-2">
				<DropdownSelect
					v-slot="{ selected }"
					v-model="sortBy"
					class="!w-auto flex-grow md:flex-grow-0"
					:name="formatMessage(messages.sortByControlName)"
					:options="['updated', 'created', 'name']"
					:display-name="formatCollectionSortOption"
				>
					<span class="font-semibold text-primary">{{
						formatMessage(commonMessages.sortByLabel)
					}}</span>
					<span class="font-semibold text-secondary">{{ selected }}</span>
				</DropdownSelect>

				<Button
					color="primary"
					class="ml-auto"
					@click="(event) => $refs.modal_creation.show(event)"
				>
					<PlusIcon aria-hidden="true" />
					{{ formatMessage(messages.createNewButton) }}
				</Button>
			</div>
		</div>
		<div class="collections-grid">
			<nuxt-link
				v-if="showFollowedProjectsCollection"
				:to="`/collection/following`"
				class="universal-card recessed collection"
			>
				<Avatar src="https://cdn.modrinth.com/follow-collection.png" size="64px" />
				<div class="details">
					<span class="title">{{ formatMessage(commonMessages.followedProjectsLabel) }}</span>
					<span class="description">
						{{ formatMessage(messages.followingCollectionDescription) }}
					</span>
					<div class="stat-bar">
						<div class="stats">
							<BoxIcon aria-hidden="true" />
							{{
								formatMessage(messages.projectsCountLabel, {
									count: formatCompactNumber(user ? user.follows.length : 0),
									countPlural: formatCompactNumberPlural(user ? user.follows.length : 0),
								})
							}}
						</div>
						<div class="stats">
							<LockIcon aria-hidden="true" />
							<span> {{ formatMessage(commonMessages.privateLabel) }} </span>
						</div>
					</div>
				</div>
			</nuxt-link>
			<nuxt-link
				v-for="collection in orderedCollections"
				:key="collection.id"
				:to="`/collection/${collection.id}`"
				class="universal-card recessed collection"
			>
				<Avatar :src="collection.icon_url" size="64px" />
				<div class="details">
					<span class="title">{{ collection.name }}</span>
					<span class="description">
						{{ collection.description }}
					</span>
					<div class="stat-bar">
						<div class="stats">
							<BoxIcon aria-hidden="true" />
							{{
								formatMessage(messages.projectsCountLabel, {
									count: formatCompactNumber(collection.projects?.length || 0),
									countPlural: formatCompactNumberPlural(collection.projects?.length || 0),
								})
							}}
						</div>
						<div class="stats">
							<template v-if="collection.status === 'listed'">
								<GlobeIcon aria-hidden="true" />
								<span> {{ formatMessage(commonMessages.publicLabel) }} </span>
							</template>
							<template v-else-if="collection.status === 'unlisted'">
								<LinkIcon aria-hidden="true" />
								<span> {{ formatMessage(commonMessages.unlistedLabel) }} </span>
							</template>
							<template v-else-if="collection.status === 'private'">
								<LockIcon aria-hidden="true" />
								<span> {{ formatMessage(commonMessages.privateLabel) }} </span>
							</template>
							<template v-else-if="collection.status === 'rejected'">
								<XIcon aria-hidden="true" />
								<span> {{ formatMessage(commonMessages.rejectedLabel) }} </span>
							</template>
						</div>
					</div>
				</div>
			</nuxt-link>
		</div>
	</div>
	<div v-if="orderedCollections.length === 0" class="empty-state-container">
		<div class="py-12 text-center">
			<BoxIcon class="mx-auto h-12 w-12 text-secondary opacity-50" aria-hidden="true" />
			<p class="mt-4 text-lg font-medium text-contrast">
				{{
					filterQuery
						? formatMessage(messages.emptyNoMatch)
						: formatMessage(messages.emptyNoCollections)
				}}
			</p>
			<p class="text-sm text-secondary">
				{{
					filterQuery
						? formatMessage(messages.emptyNoMatchHint)
						: formatMessage(messages.emptyGetStartedHint)
				}}
			</p>
		</div>
	</div>
</template>
<script setup>
import {
	BoxIcon,
	GlobeIcon,
	LinkIcon,
	LockIcon,
	PlusIcon,
	SearchIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	commonMessages,
	defineMessages,
	DropdownSelect,
	injectModrinthClient,
	StyledInput,
	useCompactNumber,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'

import CollectionCreateModal from '~/components/ui/create/CollectionCreateModal.vue'

const { formatMessage } = useVIntl()
const { formatCompactNumber, formatCompactNumberPlural } = useCompactNumber()

const messages = defineMessages({
	createNewButton: {
		id: 'dashboard.collections.button.create-new',
		defaultMessage: 'Create new',
	},
	collectionsLongTitle: {
		id: 'dashboard.collections.long-title',
		defaultMessage: 'Your collections',
	},
	followingCollectionDescription: {
		id: 'collection.description.following',
		defaultMessage: "Auto-generated collection of all the projects you're following.",
	},
	projectsCountLabel: {
		id: 'dashboard.collections.label.projects-count',
		defaultMessage: '{count} {countPlural, plural, one {project} other {projects}}',
	},
	searchInputLabel: {
		id: 'dashboard.collections.label.search-input',
		defaultMessage: 'Search your collections',
	},
	searchCollectionsPlaceholder: {
		id: 'dashboard.collections.placeholder.search',
		defaultMessage: 'Search collections...',
	},
	sortByControlName: {
		id: 'dashboard.collections.sort.control-name',
		defaultMessage: 'Sort by',
	},
	sortRecentlyUpdated: {
		id: 'dashboard.collections.sort.recently-updated',
		defaultMessage: 'Recently Updated',
	},
	sortRecentlyCreated: {
		id: 'dashboard.collections.sort.recently-created',
		defaultMessage: 'Recently Created',
	},
	sortNameAscending: {
		id: 'dashboard.collections.sort.name-ascending',
		defaultMessage: 'Name (A-Z)',
	},
	emptyNoMatch: {
		id: 'dashboard.collections.empty.no-match',
		defaultMessage: 'No collections match your search',
	},
	emptyNoCollections: {
		id: 'dashboard.collections.empty.no-collections',
		defaultMessage: "You don't have any collections yet",
	},
	emptyNoMatchHint: {
		id: 'dashboard.collections.empty.no-match-hint',
		defaultMessage: 'Try adjusting your filters or search terms.',
	},
	emptyGetStartedHint: {
		id: 'dashboard.collections.empty.get-started-hint',
		defaultMessage: 'Create your first collection to get started!',
	},
})

definePageMeta({
	middleware: 'auth',
})

useHead({
	title: () => `${formatMessage(messages.collectionsLongTitle)} - Modrinth`,
})

const auth = await useAuth()
const user = await useUser()
const client = injectModrinthClient()

if (import.meta.client) {
	await initUserFollows()
}

const filterQuery = ref('')

const { data: collections } = useQuery({
	queryKey: ['user', auth.value.user.id, 'collections'],
	queryFn: () => client.labrinth.users_v2.getCollections(auth.value.user.id),
})

const route = useNativeRoute()
const router = useNativeRouter()
const validSortOptions = ['updated', 'created', 'name']
const sortBy = ref(validSortOptions.includes(route.query.s) ? route.query.s : 'updated')

const showFollowedProjectsCollection = computed(() =>
	formatMessage(commonMessages.followedProjectsLabel)
		.toLowerCase()
		.includes(filterQuery.value.toLowerCase()),
)

function formatCollectionSortOption(option) {
	if (option === 'updated') {
		return formatMessage(messages.sortRecentlyUpdated)
	}

	if (option === 'created') {
		return formatMessage(messages.sortRecentlyCreated)
	}

	return formatMessage(messages.sortNameAscending)
}

const orderedCollections = computed(() => {
	if (!collections.value) return []
	return [...collections.value]
		.filter(
			(c) => !filterQuery.value || c.name.toLowerCase().includes(filterQuery.value.toLowerCase()),
		)
		.sort((a, b) => {
			if (sortBy.value === 'name') return a.name.localeCompare(b.name)
			if (sortBy.value === 'created') return new Date(b.created) - new Date(a.created)
			return new Date(b.updated) - new Date(a.updated)
		})
})

watch(sortBy, (newVal) => {
	router.replace({
		path: route.path,
		query: {
			...route.query,
			s: newVal,
		},
	})
})
</script>
<style lang="scss">
.collections-grid {
	display: grid;
	grid-template-columns: repeat(2, 1fr);

	@media screen and (max-width: 800px) {
		grid-template-columns: repeat(1, 1fr);
	}

	gap: var(--gap-md);

	.collection {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: var(--gap-md);
		margin-bottom: 0;

		.details {
			display: flex;
			flex-direction: column;
			gap: var(--gap-sm);

			.title {
				color: var(--color-contrast);
				font-weight: 600;
				font-size: var(--font-size-md);
			}

			.description {
				color: var(--color-secondary);
				font-size: var(--font-size-sm);

				word-break: break-word;
			}

			.stat-bar {
				display: flex;
				align-items: center;
				gap: var(--gap-md);
				margin-top: auto;
			}

			.stats {
				display: flex;
				align-items: center;
				gap: var(--gap-xs);

				svg {
					color: var(--color-secondary);
				}
			}
		}
	}
}
</style>
