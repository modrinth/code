<template>
	<div v-if="isLoading" class="flex min-h-[50vh] items-center justify-center">
		<SpinnerIcon class="h-12 w-12 animate-spin text-brand" />
	</div>
	<template v-else-if="collection && creator">
		<ConfirmModal
			v-if="canEdit"
			ref="deleteModal"
			:title="formatMessage(messages.deleteModalTitle)"
			:description="formatMessage(messages.deleteModalDescription)"
			:has-to-type="false"
			:proceed-label="formatMessage(commonMessages.deleteLabel)"
			@proceed="deleteCollection()"
		/>
		<NewModal v-if="canEdit" ref="editModal" :header="formatMessage(messages.editingCollection)">
			<div class="flex w-[30rem] flex-col gap-3">
				<div class="flow-root">
					<div class="group relative float-end ml-4">
						<OverflowMenu
							v-tooltip="formatMessage(messages.editIconButton)"
							:dropdown-id="`${baseId}-edit-icon`"
							class="m-0 cursor-pointer appearance-none border-none bg-transparent p-0 transition-transform group-active:scale-95"
							:options="[
								{
									id: 'select',
									action: () => {
										const input = iconInputRef?.$el?.querySelector('input')
										input?.click()
									},
								},
								{
									id: 'remove',
									color: 'danger',
									action: () => {
										deletedIcon = true
										previewImage = null
									},
									shown: !deletedIcon && (previewImage || collection.icon_url),
								},
							]"
						>
							<Avatar
								:src="deletedIcon ? null : previewImage ? previewImage : collection.icon_url"
								size="108px"
								class="!border-4 group-hover:brightness-75"
								no-shadow
							/>
							<div class="absolute right-0 top-0 m-2">
								<div
									class="m-0 flex aspect-square items-center justify-center rounded-full border-[1px] border-solid border-button-border bg-button-bg p-2 text-primary"
								>
									<EditIcon aria-hidden="true" class="h-4 w-4 text-primary" />
								</div>
							</div>
							<template #select>
								<UploadIcon />
								{{
									previewImage || collection.icon_url
										? formatMessage(messages.replaceIcon)
										: formatMessage(messages.selectIcon)
								}}
							</template>
							<template #remove>
								<XIcon />
								{{ formatMessage(messages.removeIconButton) }}
							</template>
						</OverflowMenu>
						<FileInput
							id="collection-icon-input"
							ref="iconInputRef"
							:max-size="262144"
							:show-icon="false"
							accept="image/png,image/jpeg,image/gif,image/webp"
							class="hidden"
							aria-label="Upload icon"
							@change="showPreviewImage"
						/>
					</div>
					<div class="overflow-hidden">
						<label class="mb-2 block text-lg font-semibold text-contrast" for="collection-title">
							{{ formatMessage(commonMessages.titleLabel) }}
						</label>
						<StyledInput
							id="collection-title"
							v-model="current.name"
							:maxlength="255"
							autocomplete="off"
							wrapper-class="w-full"
						/>
					</div>
					<label
						class="mb-2 mt-4 block text-lg font-semibold text-contrast"
						for="collection-description"
					>
						{{ formatMessage(commonMessages.descriptionLabel) }}
					</label>
					<StyledInput
						id="collection-description"
						v-model="current.description"
						multiline
						:maxlength="255"
						wrapper-class="h-24"
					/>
					<label for="visibility" class="mb-2 mt-4 block text-lg font-semibold text-contrast">
						{{ formatMessage(commonMessages.visibilityLabel) }}
					</label>
					<RadioButtons v-model="current.status" :items="['listed', 'unlisted', 'private']">
						<template #default="{ item }">
							<span class="flex items-center gap-1">
								{{
									item === 'listed'
										? formatMessage(commonMessages.publicLabel)
										: formatMessage(commonMessages[`${item}Label`])
								}}
							</span>
						</template>
					</RadioButtons>
				</div>
				<div class="flex justify-end gap-2">
					<ButtonStyled>
						<button class="w-24" @click="() => editModal?.hide()">
							<XIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.cancelButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button class="w-36" :disabled="saving" @click="save()">
							<SpinnerIcon v-if="saving" class="animate-spin" aria-hidden="true" />
							<SaveIcon v-else aria-hidden="true" />
							{{
								saving
									? formatMessage(commonMessages.savingButton)
									: formatMessage(commonMessages.saveButton)
							}}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</NewModal>
		<NormalPage :sidebar="cosmetics.leftContentLayout ? 'left' : 'right'">
			<template #header>
				<div class="flex flex-col gap-4">
					<ClientOnly>
						<nuxt-link
							v-if="returnLink"
							:to="returnLink.link"
							class="flex w-fit items-center gap-1 text-brand-blue hover:underline"
						>
							<ChevronLeftIcon aria-hidden="true" />
							{{ formatMessage(returnLink.message, { user: creator.username }) }}
						</nuxt-link>
					</ClientOnly>
					<div class="grid grid-cols-[auto_1fr] gap-4 sm:grid-cols-[auto_1fr_auto]">
						<Avatar :src="collection.icon_url" size="64px" />
						<div class="flex flex-col gap-3">
							<h1 class="heading-2xl">
								{{ collection.name }}
							</h1>
							<div class="flex items-center gap-2">
								<template v-if="canEdit || collection.status !== 'listed'">
									<div class="flex items-center gap-1">
										<template v-if="collection.status === 'listed'">
											<GlobeIcon aria-hidden="true" />
											{{ formatMessage(commonMessages.publicLabel) }}
										</template>
										<template v-else-if="collection.status === 'unlisted'">
											<LinkIcon aria-hidden="true" />
											{{ formatMessage(commonMessages.unlistedLabel) }}
										</template>
										<template v-else-if="collection.status === 'private'">
											<LockIcon aria-hidden="true" />
											{{ formatMessage(commonMessages.privateLabel) }}
										</template>
										<template v-else-if="collection.status === 'rejected'">
											<XIcon aria-hidden="true" />
											{{ formatMessage(commonMessages.rejectedLabel) }}
										</template>
									</div>
									<span class="text-secondary">•</span>
								</template>
								<span>
									<IntlFormatted
										:message-id="messages.projectsCountLabel"
										:values="{
											count: formatCompactNumber(projects?.length || 0),
											type: formatMessage(
												getProjectTypeSentenceMessage(
													projectTypes.length === 1 ? projectTypes[0] : 'project',
												),
												{
													count: formatCompactNumberPlural(projects?.length || 0),
												},
											),
										}"
									>
										<template #stat="{ children }">
											<span>
												<component :is="() => normalizeChildren(children)" />
											</span>
										</template>
									</IntlFormatted>
								</span>
							</div>
						</div>
						<div class="col-span-2 flex items-center gap-2 sm:col-span-1">
							<template v-if="canEdit">
								<ButtonStyled size="large">
									<button @click="openEditModal">
										<EditIcon aria-hidden="true" />
										{{ formatMessage(commonMessages.editButton) }}
									</button>
								</ButtonStyled>
								<ButtonStyled size="large" circular type="transparent">
									<OverflowMenu
										:dropdown-id="`${baseId}-more-options`"
										:options="[
											{
												id: 'delete',
												color: 'red',
												action: () => deleteModal?.show(),
											},
										]"
										:aria-label="formatMessage(commonMessages.moreOptionsButton)"
									>
										<MoreVerticalIcon aria-hidden="true" />
										<template #delete>
											<TrashIcon aria-hidden="true" />
											{{ formatMessage(commonMessages.deleteLabel) }}
										</template>
									</OverflowMenu>
								</ButtonStyled>
							</template>
						</div>
					</div>
					<HorizontalRule />
				</div>
			</template>
			<template #sidebar>
				<SidebarCard
					v-if="collection.description"
					:title="formatMessage(commonMessages.descriptionLabel)"
				>
					<div
						v-if="supportsMarkdown"
						class="description-body"
						v-html="renderString(collection.description)"
					/>
					<p v-else class="m-0 break-words">{{ collection.description }}</p>
				</SidebarCard>
				<SidebarCard
					v-if="collection.id !== 'following'"
					:title="formatMessage(messages.curatedByLabel)"
				>
					<nuxt-link
						class="group flex w-fit items-center gap-2 leading-[1.2] text-primary"
						:to="`/user/${creator.username}`"
					>
						<Avatar :src="creator.avatar_url" :alt="creator.username" size="32px" circle />
						<div class="flex flex-col">
							<span class="flex w-full flex-nowrap items-center gap-1 group-hover:underline">
								<span class="min-w-0 overflow-hidden truncate">{{ creator.username }}</span>
							</span>
						</div>
					</nuxt-link>
				</SidebarCard>
				<AdPlaceholder v-if="!auth.user" />
				<SidebarCard
					v-if="collection.id !== 'following'"
					:title="formatMessage(commonMessages.detailsLabel)"
				>
					<div class="flex flex-col gap-2">
						<span
							v-tooltip="formatDateTime(collection.created)"
							class="flex w-fit items-center gap-2"
						>
							<CalendarIcon aria-hidden="true" />
							{{
								formatMessage(messages.createdAtLabel, {
									ago: formatRelativeTime(collection.created),
								})
							}}
						</span>
						<span
							v-if="showUpdatedDate"
							v-tooltip="formatDateTime(collection.updated)"
							class="flex w-fit items-center gap-2"
						>
							<UpdatedIcon aria-hidden="true" />
							{{
								formatMessage(messages.updatedAtLabel, {
									ago: formatRelativeTime(collection.updated),
								})
							}}
						</span>
					</div>
				</SidebarCard>
			</template>
			<NavTabs
				v-if="projects && projectTypes.length > 1"
				:links="[
					{
						label: formatMessage(commonMessages.allProjectType),
						href: `/collection/${collection.id}`,
					},
					...projectTypes.map((x) => {
						return {
							label: formatMessage(getProjectTypeCategoryMessage(x)),
							href: `/collection/${collection.id}/${x}s`,
						}
					}),
				]"
				class="mb-4"
			/>

			<template v-if="projects && projects.length > 0">
				<div class="mb-4 flex flex-col gap-3">
					<div class="flex flex-wrap items-center gap-2">
						<StyledInput
							v-model="searchQuery"
							:icon="SearchIcon"
							type="text"
							autocomplete="off"
							:placeholder="formatMessage(messages.searchPlaceholder)"
							clearable
							wrapper-class="w-full flex-grow sm:w-auto"
						/>
						<Combobox
							v-model="currentSort"
							:options="sortOptions"
							class="!w-[14rem] min-w-max max-w-full flex-grow sm:flex-grow-0"
						>
							<template #prefix>
								<span class="font-semibold text-primary">
									{{ formatMessage(commonMessages.sortByLabel) }}
								</span>
							</template>
						</Combobox>
						<ButtonStyled v-if="collectionFilterTypes.length > 0">
							<button @click="filtersOpen = !filtersOpen">
								<FilterIcon aria-hidden="true" />
								{{ formatMessage(commonMessages.filtersLabel) }}
								<span
									v-if="selectedFilters.length > 0"
									class="flex h-5 min-w-5 items-center justify-center rounded-full bg-brand px-1 text-xs font-bold text-brand-inverted"
								>
									{{ selectedFilters.length }}
								</span>
								<DropdownIcon
									aria-hidden="true"
									class="h-4 w-4 transition-transform"
									:class="{ 'rotate-180': filtersOpen }"
								/>
							</button>
						</ButtonStyled>
					</div>
					<div
						v-if="filtersOpen && collectionFilterTypes.length > 0"
						class="grid grid-cols-1 items-start gap-3 sm:grid-cols-2 lg:grid-cols-3"
					>
						<SearchSidebarFilter
							v-for="filterType in collectionFilterTypes"
							:key="filterType.id"
							v-model:selected-filters="selectedFilters"
							v-model:toggled-groups="toggledGroups"
							:filter-type="filterType"
							:provided-filters="[]"
							:open-by-default="false"
							class="card-shadow rounded-2xl border border-solid bg-surface-3 border-surface-4"
							button-class="button-animation flex flex-col gap-1 px-4 py-3 w-full bg-transparent cursor-pointer border-none"
							content-class="mb-4 mx-3"
							inner-panel-class="p-1"
						>
							<template #header>
								<h3 class="m-0 text-base font-semibold text-contrast">
									{{ filterType.formatted_name }}
								</h3>
							</template>
						</SearchSidebarFilter>
					</div>
					<SearchFilterControl
						v-model:selected-filters="selectedFilters"
						:filters="collectionFilterTypes"
						:provided-filters="[]"
						:overridden-provided-filter-types="[]"
					/>
				</div>
			<ProjectCardList
					v-if="displayProjects.length > 0"
				:layout="cosmetics.searchDisplayMode.collection"
			>
				<ProjectCard
						v-for="project in displayProjects"
					:key="project.id"
					:link="`/${project.project_type}/${project.slug ?? project.id}`"
					:title="project.title"
					:icon-url="project.icon_url"
					:banner="project.gallery.find((element) => element.featured)?.url"
					:summary="project.description"
					:date-updated="project.updated"
					:downloads="project.downloads ?? 0"
					:followers="project.followers ?? 0"
					:tags="project.categories"
					:environment="{
						clientSide: project.client_side,
						serverSide: project.server_side,
					}"
					:color="project.color"
					:layout="
						cosmetics.searchDisplayMode.collection === 'grid' ||
						cosmetics.searchDisplayMode.collection === 'gallery'
							? 'grid'
							: 'list'
					"
				>
					<template v-if="canEdit || collection.id === 'following'" #actions>
						<ButtonStyled v-if="canEdit">
							<button class="remove-btn" :disabled="removing" @click="() => removeProject(project)">
								<SpinnerIcon v-if="removing" class="animate-spin" aria-hidden="true" />
								<XIcon v-else aria-hidden="true" />
								{{ formatMessage(messages.removeProjectButton) }}
							</button>
						</ButtonStyled>
						<ButtonStyled v-if="collection.id === 'following'">
							<button @click="unfollowProject(project)">
								<HeartMinusIcon aria-hidden="true" />
								{{ formatMessage(messages.unfollowProjectButton) }}
							</button>
						</ButtonStyled>
					</template>
				</ProjectCard>
			</ProjectCardList>
				<EmptyState
					v-else
					type="no-search-result"
					:heading="formatMessage(messages.noResultsLabel)"
				>
					<template #actions>
						<ButtonStyled v-if="searchQuery || selectedFilters.length > 0">
							<button @click="clearSearchAndFilters">
								<XIcon aria-hidden="true" />
								{{ formatMessage(messages.clearFiltersButton) }}
							</button>
						</ButtonStyled>
					</template>
				</EmptyState>
			</template>
			<EmptyState v-else type="empty-inbox" :heading="formatMessage(messages.noProjectsLabel)">
				<template #actions>
					<ButtonStyled v-if="auth.user && auth.user.id === creator.id" color="brand">
						<nuxt-link class="mx-auto w-min" to="/discover/mods">
							<CompassIcon class="size-5" />
							Discover mods
						</nuxt-link>
					</ButtonStyled>
				</template>
			</EmptyState>
		</NormalPage>
	</template>
</template>

<script setup>
import {
	CalendarIcon,
	ChevronLeftIcon,
	CompassIcon,
	DropdownIcon,
	EditIcon,
	FilterIcon,
	getCategoryIcon,
	getLoaderIcon,
	GlobeIcon,
	HeartMinusIcon,
	LinkIcon,
	LockIcon,
	MoreVerticalIcon,
	SaveIcon,
	SearchIcon,
	SpinnerIcon,
	TrashIcon,
	UpdatedIcon,
	UploadIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	Combobox,
	commonMessages,
	commonProjectTypeCategoryMessages,
	commonProjectTypeSentenceMessages,
	ConfirmModal,
	defineMessage,
	defineMessages,
	EmptyState,
	FileInput,
	formatCategory,
	formatCategoryHeader,
	formatLoader,
	HorizontalRule,
	injectModrinthClient,
	injectNotificationManager,
	IntlFormatted,
	NavTabs,
	NewModal,
	normalizeChildren,
	NormalPage,
	OverflowMenu,
	ProjectCard,
	ProjectCardList,
	RadioButtons,
	SearchFilterControl,
	SearchSidebarFilter,
	SidebarCard,
	StyledInput,
	useCompactNumber,
	useFormatDateTime,
	useRelativeTime,
	useSavable,
	useVIntl,
} from '@modrinth/ui'
import { isAdmin, renderString } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import { onServerPrefetch } from 'vue'

import AdPlaceholder from '~/components/ui/AdPlaceholder.vue'

const { handleError } = injectNotificationManager()
const api = injectModrinthClient()
const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const { formatCompactNumber, formatCompactNumberPlural } = useCompactNumber()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})

const route = useNativeRoute()
const router = useRouter()
const auth = await useAuth()
const cosmetics = useCosmetics()
const tags = useGeneratedState()
const queryClient = useQueryClient()
const baseId = useId()

async function fetchProjectsByIds(projectIds) {
	const segmentSize = 800
	const segments = []
	for (let i = 0; i < projectIds.length; i += segmentSize) {
		segments.push(projectIds.slice(i, i + segmentSize))
	}
	const results = await Promise.all(
		segments.map((ids) => api.labrinth.projects_v2.getMultiple(ids)),
	)
	const projects = results.flat()
	for (const project of projects) {
		project.categories = project.categories.concat(project.loaders)
	}
	return projects
}

async function fetchFollowedProjects(userId) {
	const projects = await api.labrinth.users_v2.getFollowedProjects(userId)
	for (const project of projects) {
		project.categories = project.categories.concat(project.loaders)
	}
	return projects
}

const messages = defineMessages({
	collectionDescription: {
		id: 'collection.description',
		defaultMessage: '{description} - View the collection {name} by {username} on Modrinth',
	},
	collectionTitle: {
		id: 'collection.title',
		defaultMessage: '{name} - Collection',
	},
	editIconButton: {
		id: 'collection.button.edit-icon',
		defaultMessage: 'Edit icon',
	},
	removeIconButton: {
		id: 'collection.button.remove-icon',
		defaultMessage: 'Remove icon',
	},
	selectIcon: {
		id: 'collection.button.select-icon',
		defaultMessage: 'Select icon',
	},
	replaceIcon: {
		id: 'collection.button.replace-icon',
		defaultMessage: 'Replace icon',
	},
	editingCollection: {
		id: 'collection.editing',
		defaultMessage: 'Editing collection',
	},
	createdAtLabel: {
		id: 'collection.label.created-at',
		defaultMessage: 'Created {ago}',
	},
	collectionNotFoundError: {
		id: 'collection.error.not-found',
		defaultMessage: 'Collection not found',
	},
	curatedByLabel: {
		id: 'collection.label.curated-by',
		defaultMessage: 'Curated by',
	},
	deleteModalDescription: {
		id: 'collection.delete-modal.description',
		defaultMessage: 'This will permanently delete this collection. This action cannot be undone.',
	},
	deleteModalTitle: {
		id: 'collection.delete-modal.title',
		defaultMessage: 'Are you sure you want to delete this collection?',
	},
	followingCollectionDescription: {
		id: 'collection.description.following',
		defaultMessage: "Auto-generated collection of all the projects you're following.",
	},
	noProjectsLabel: {
		id: 'collection.label.no-projects',
		defaultMessage: 'No projects in collection yet',
	},
	noResultsLabel: {
		id: 'collection.label.no-results',
		defaultMessage: 'No projects match your search',
	},
	clearFiltersButton: {
		id: 'collection.button.clear-filters',
		defaultMessage: 'Clear filters',
	},
	searchPlaceholder: {
		id: 'collection.search.placeholder',
		defaultMessage: 'Search collection...',
	},
	gameVersionFilterLabel: {
		id: 'search.filter_type.game_version',
		defaultMessage: 'Game version',
	},
	loaderFilterLabel: {
		id: 'search.filter_type.mod_loader',
		defaultMessage: 'Loader',
	},
	showAllVersionsLabel: {
		id: 'search.filter_type.game_version.all_versions',
		defaultMessage: 'Show all versions',
	},
	projectsCountLabel: {
		id: 'collection.label.projects-count',
		defaultMessage: '{count, plural, =0 {No projects yet} other {<stat>{count}</stat> {type}}}',
	},
	removeProjectButton: {
		id: 'collection.button.remove-project',
		defaultMessage: 'Remove project',
	},
	unfollowProjectButton: {
		id: 'collection.button.unfollow-project',
		defaultMessage: 'Unfollow project',
	},
	updatedAtLabel: {
		id: 'collection.label.updated-at',
		defaultMessage: 'Updated {ago}',
	},
})

const returnLink = computed(() => {
	const from = router.options?.history?.state?.back
	if (from?.startsWith('/dashboard/collections')) {
		return {
			link: from,
			message: defineMessage({
				id: 'collection.return-link.dashboard-collections',
				defaultMessage: 'Your collections',
			}),
		}
	} else if (from?.startsWith('/user/')) {
		return {
			link: from,
			message: defineMessage({
				id: 'collection.return-link.user',
				defaultMessage: `{user}'s profile`,
			}),
		}
	}
	return null
})

const collectionId = useRouteId('collection')
const isFollowingCollection = computed(() => collectionId === 'following')

// Static collection for "following" page
const followingCollection = computed(() =>
	isFollowingCollection.value
		? {
				id: 'following',
				icon_url: 'https://cdn.modrinth.com/follow-collection.png',
				name: formatMessage(commonMessages.followedProjectsLabel),
				description: formatMessage(messages.followingCollectionDescription),
				status: 'private',
				user: auth.value.user?.id,
				created: auth.value.user?.created,
				updated: auth.value.user?.created,
			}
		: null,
)

// Query for regular collections
const {
	data: fetchedCollection,
	refetch: refreshFetchedCollection,
	error: collectionError,
	isPending: collectionIsPending,
} = useQuery({
	queryKey: computed(() => ['collection', collectionId]),
	queryFn: () => api.labrinth.collections.get(collectionId),
	enabled: computed(() => !!collectionId && !isFollowingCollection.value),
})

watch(
	collectionError,
	(error) => {
		if (error && collectionId && !isFollowingCollection.value) {
			const status = error.statusCode ?? error.status ?? 404
			showError({
				fatal: true,
				statusCode: status,
				message: formatMessage(messages.collectionNotFoundError),
			})
		}
	},
	{ immediate: true },
)

// Unified collection ref
const collection = computed(() => followingCollection.value ?? fetchedCollection.value)
const refreshCollection = async () => {
	if (!isFollowingCollection.value) {
		await refreshFetchedCollection()
	}
}

// Query for creator (only for regular collections)
const { data: fetchedCreator, isPending: creatorIsPending } = useQuery({
	queryKey: computed(() => ['user', collection.value?.user]),
	queryFn: () => api.labrinth.users_v2.get(collection.value.user),
	enabled: computed(() => !isFollowingCollection.value && !!collection.value?.user),
})

// Unified creator ref
const creator = computed(() =>
	isFollowingCollection.value ? auth.value.user : fetchedCreator.value,
)

const supportsMarkdown = computed(() => creator.value?.id === '2REoufqX')

// Query for followed projects
const {
	data: followedProjects,
	refetch: refetchFollowedProjects,
	isFetching: followedProjectsIsFetching,
} = useQuery({
	queryKey: computed(() => ['user', auth.value.user?.id, 'follows']),
	queryFn: async () => fetchFollowedProjects(auth.value.user.id),
	enabled: computed(() => isFollowingCollection.value && !!auth.value.user?.id),
	placeholderData: [],
})

// Query for collection projects
const {
	data: collectionProjects,
	refetch: refetchCollectionProjects,
	isFetching: collectionProjectsIsFetching,
} = useQuery({
	queryKey: computed(() => ['projects', collection.value?.projects]),
	queryFn: () => fetchProjectsByIds(collection.value.projects),
	enabled: computed(() => !isFollowingCollection.value && !!collection.value?.projects?.length),
	placeholderData: [],
})

// Unified projects ref
const projects = computed(() =>
	isFollowingCollection.value ? followedProjects.value : collectionProjects.value,
)

// Loading state
const isLoading = computed(() => {
	if (!import.meta.client) return false

	if (isFollowingCollection.value) {
		return followedProjectsIsFetching.value
	}
	return collectionIsPending.value || creatorIsPending.value || collectionProjectsIsFetching.value
})

onServerPrefetch(async () => {
	if (isFollowingCollection.value) {
		const userId = auth.value.user?.id
		if (!userId) return

		await queryClient.ensureQueryData({
			queryKey: ['user', userId, 'follows'],
			queryFn: () => fetchFollowedProjects(userId),
		})
		return
	}

	if (!collectionId) return

	const collectionData = await queryClient.ensureQueryData({
		queryKey: ['collection', collectionId],
		queryFn: () => api.labrinth.collections.get(collectionId),
	})

	if (collectionData?.user) {
		await queryClient.ensureQueryData({
			queryKey: ['user', collectionData.user],
			queryFn: () => api.labrinth.users_v2.get(collectionData.user),
		})
	}

	if (collectionData?.projects?.length) {
		await queryClient.ensureQueryData({
			queryKey: ['projects', collectionData.projects],
			queryFn: () => fetchProjectsByIds(collectionData.projects),
		})
	}
})

watch(
	[collection, creator],
	([col, cre]) => {
		if (col && cre) {
			const canonicalUrl = col ? `https://modrinth.com/collection/${col.id}` : undefined
			useSeoMeta({
				title: formatMessage(messages.collectionTitle, { name: col.name }),
				description: formatMessage(messages.collectionDescription, {
					name: col.name,
					description: col.description,
					username: cre.username,
				}),
				ogTitle: formatMessage(messages.collectionTitle, { name: col.name }),
				ogDescription: col.description,
				ogImage: col.icon_url ?? 'https://cdn.modrinth.com/placeholder.png',
				ogUrl: canonicalUrl,
				robots: col.status === 'listed' ? 'all' : 'noindex',
			})
			useHead({
				link: [
					{
						rel: 'canonical',
						href: canonicalUrl,
					},
				],
			})
		}
	},
	{ immediate: true },
)

const canEdit = computed(
	() =>
		collection.value &&
		auth.value.user &&
		(auth.value.user.id === collection.value.user || isAdmin(auth.value.user)) &&
		collection.value.id !== 'following',
)

const projectTypes = computed(() => {
	const projectSet = new Set(
		projects.value?.map((project) => project?.project_type).filter((x) => x !== undefined) || [],
	)
	projectSet.delete('project')
	return Array.from(projectSet)
})

function getProjectTypeSentenceMessage(type) {
	return commonProjectTypeSentenceMessages[type] ?? commonProjectTypeSentenceMessages.project
}

function getProjectTypeCategoryMessage(type) {
	return commonProjectTypeCategoryMessages[type] ?? commonProjectTypeCategoryMessages.project
}

const searchQuery = ref('')
const filtersOpen = ref(false)
const selectedFilters = ref([])
const toggledGroups = ref([])

const sortOptions = [
	{ value: 'downloads', label: 'Downloads' },
	{ value: 'follows', label: 'Followers' },
	{ value: 'updated', label: 'Date updated' },
	{ value: 'newest', label: 'Date published' },
	{ value: 'name', label: 'Name' },
]
const currentSort = ref('downloads')

const typeFilteredProjects = computed(() => {
	if (!projects.value) return []
	const typeParam = route.params.projectType
	if (typeParam === undefined) return projects.value
	const type = typeParam.substring(0, typeParam.length - 1)
	return projects.value.filter((x) => x.project_type === type)
})

const collectionFilterTypes = computed(() => {
	const gameVersions = new Set()
	const loaders = new Set()
	const categories = new Set()
	for (const project of typeFilteredProjects.value) {
		project.game_versions?.forEach((version) => gameVersions.add(version))
		project.loaders?.forEach((loader) => loaders.add(loader))
		project.categories?.forEach((category) => categories.add(category))
	}
	for (const loader of loaders) {
		categories.delete(loader)
	}

	const filterTypes = []

	const gameVersionOptions = tags.value.gameVersions
		.filter((gameVersion) => gameVersions.has(gameVersion.version))
		.map((gameVersion) => ({
			id: gameVersion.version,
			toggle_group: gameVersion.version_type !== 'release' ? 'all_versions' : undefined,
			method: 'or',
			value: gameVersion.version,
		}))
	if (gameVersionOptions.length > 0) {
		filterTypes.push({
			id: 'game_version',
			formatted_name: formatMessage(messages.gameVersionFilterLabel),
			supported_project_types: [],
			display: 'scrollable',
			query_param: 'v',
			supports_negative_filter: false,
			searchable: true,
			toggle_groups: gameVersionOptions.some((option) => option.toggle_group)
				? [{ id: 'all_versions', formatted_name: formatMessage(messages.showAllVersionsLabel) }]
				: [],
			options: gameVersionOptions,
		})
	}

	const loaderOptions = tags.value.loaders
		.filter((loader) => loaders.has(loader.name))
		.map((loader) => ({
			id: loader.name,
			formatted_name: formatLoader(formatMessage, loader.name),
			icon: getLoaderIcon(loader.name),
			method: 'or',
			value: loader.name,
		}))
	if (loaderOptions.length > 0) {
		filterTypes.push({
			id: 'mod_loader',
			formatted_name: formatMessage(messages.loaderFilterLabel),
			supported_project_types: [],
			display: 'scrollable',
			query_param: 'g',
			supports_negative_filter: true,
			searchable: false,
			options: loaderOptions,
		})
	}

	const seenCategories = new Set()
	const categoryOptions = []
	for (const category of tags.value.categories) {
		if (!categories.has(category.name) || seenCategories.has(category.name)) continue
		seenCategories.add(category.name)
		categoryOptions.push({
			id: category.name,
			formatted_name: formatCategory(formatMessage, category.name),
			icon: getCategoryIcon(category.name),
			method: 'or',
			value: category.name,
		})
	}
	if (categoryOptions.length > 0) {
		filterTypes.push({
			id: 'category',
			formatted_name: formatCategoryHeader(formatMessage, 'categories'),
			supported_project_types: [],
			display: 'scrollable',
			query_param: 'f',
			supports_negative_filter: true,
			searchable: false,
			options: categoryOptions,
		})
	}

	return filterTypes
})

watch(
	() => route.params.projectType,
	() => {
		const validOptions = new Set(
			collectionFilterTypes.value.flatMap((filterType) =>
				filterType.options.map((option) => `${filterType.id}:${option.id}`),
			),
		)
		selectedFilters.value = selectedFilters.value.filter((filter) =>
			validOptions.has(`${filter.type}:${filter.option}`),
		)
	},
)

function projectMatchesFilters(project) {
	for (const filterType of collectionFilterTypes.value) {
		const selected = selectedFilters.value.filter((filter) => filter.type === filterType.id)
		if (selected.length === 0) continue
		const values =
			filterType.id === 'game_version'
				? (project.game_versions ?? [])
				: filterType.id === 'mod_loader'
					? (project.loaders ?? [])
					: (project.categories ?? [])
		if (selected.some((filter) => filter.negative && values.includes(filter.option))) {
			return false
		}
		const included = selected.filter((filter) => !filter.negative)
		if (included.length > 0 && !included.some((filter) => values.includes(filter.option))) {
			return false
		}
	}
	return true
}

const displayProjects = computed(() => {
	const query = searchQuery.value.trim().toLowerCase()
	const filtered = typeFilteredProjects.value.filter(
		(project) =>
			(!query ||
				project.title?.toLowerCase().includes(query) ||
				project.description?.toLowerCase().includes(query)) &&
			projectMatchesFilters(project),
	)
	return filtered.sort((a, b) => {
		switch (currentSort.value) {
			case 'follows':
				return (b.followers ?? 0) - (a.followers ?? 0)
			case 'updated':
				return dayjs(b.updated).diff(dayjs(a.updated))
			case 'newest':
				return dayjs(b.published).diff(dayjs(a.published))
			case 'name':
				return a.title.localeCompare(b.title)
			default:
				return (b.downloads ?? 0) - (a.downloads ?? 0)
		}
	})
})

function clearSearchAndFilters() {
	searchQuery.value = ''
	selectedFilters.value = []
}

const showUpdatedDate = computed(() => {
	if (!collection.value?.updated || !collection.value?.created) {
		return false
	}
	return dayjs(collection.value.updated).diff(dayjs(collection.value.created), 'minute') > 1
})

const editModal = ref(null)
const deleteModal = ref(null)
const iconInputRef = ref(null)
const icon = ref(null)
const deletedIcon = ref(false)
const previewImage = ref(null)
const saving = ref(false)
const removing = ref(false)

const {
	saved,
	current,
	reset,
	save: saveCollection,
} = useSavable(
	() => ({
		name: collection.value?.name ?? '',
		description: collection.value?.description || null,
		status: collection.value?.status ?? 'private',
	}),
	async (changes) => {
		saving.value = true
		startLoading()
		try {
			if (deletedIcon.value) {
				await api.labrinth.collections.deleteIcon(collection.value.id)
			} else if (icon.value) {
				const ext = icon.value?.type?.split('/').pop()
				if (!ext) throw new Error('Invalid file type')
				await api.labrinth.collections.editIcon(collection.value.id, icon.value, ext)
			}

			if (Object.keys(changes).length > 0) {
				await api.labrinth.collections.edit(collection.value.id, changes)
			}

			await refreshCollection().then(reset)

			icon.value = null
			deletedIcon.value = false
			previewImage.value = null

			editModal.value?.hide()
		} catch (err) {
			handleError(err)
		} finally {
			await initUserCollections()
			stopLoading()
			saving.value = false
		}
	},
)

async function unfollowProject(project) {
	await userFollowProject(project)
	await refetchFollowedProjects()
}

async function removeProject(project) {
	removing.value = true
	startLoading()
	try {
		const currentProjects = collection.value.projects || []
		const updatedProjects = currentProjects.filter((id) => id !== project.id)

		await api.labrinth.collections.edit(collection.value.id, {
			new_projects: updatedProjects,
		})

		await refreshCollection()
		await refetchCollectionProjects()
		await initUserCollections()
	} catch (err) {
		handleError(err)
	} finally {
		removing.value = false
		stopLoading()
	}
}

const save = async () => {
	const hasIconChanges = deletedIcon.value || icon.value
	const hasCollectionChanges = computed(() => {
		const keys = Object.keys(current.value)
		for (const key of keys) {
			if (saved.value[key] !== current.value[key]) {
				return true
			}
		}
		return false
	}).value

	if (!hasCollectionChanges && !hasIconChanges) {
		return
	}

	if (hasCollectionChanges) {
		saveCollection()
	} else {
		saving.value = true
		startLoading()
		try {
			if (deletedIcon.value) {
				await api.labrinth.collections.deleteIcon(collection.value.id)
			} else if (icon.value) {
				const ext = icon.value?.type?.split('/').pop()
				if (!ext) throw new Error('Invalid file type')
				await api.labrinth.collections.editIcon(collection.value.id, icon.value, ext)
			}

			await refreshCollection().then(reset)

			icon.value = null
			deletedIcon.value = false
			previewImage.value = null

			editModal.value?.hide()
		} catch (err) {
			handleError(err)
		} finally {
			await initUserCollections()
			stopLoading()
			saving.value = false
		}
	}
}

async function deleteCollection() {
	startLoading()
	try {
		await api.labrinth.collections.delete(collection.value.id)
		if (auth.value.user.id === collection.value.user) {
			await navigateTo('/dashboard/collections')
		} else {
			await navigateTo(`/user/${collection.value.user}/collections`)
		}
	} catch (err) {
		handleError(err)
	}
	await initUserCollections()
	stopLoading()
}

function showPreviewImage(files) {
	const reader = new FileReader()
	icon.value = files[0]
	deletedIcon.value = false
	reader.readAsDataURL(icon.value)
	reader.onload = (event) => {
		previewImage.value = event.target.result
	}
}

function openEditModal(event) {
	reset()
	icon.value = null
	deletedIcon.value = false
	previewImage.value = null
	editModal.value?.show(event)
}
</script>

<style scoped lang="scss">
:deep(.description-body) {
	p {
		margin: 0;
	}

	a {
		color: var(--color-brand);
		font-weight: 600;
	}

	a:hover {
		text-decoration: underline;
	}
}
</style>
