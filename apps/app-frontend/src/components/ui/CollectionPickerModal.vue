<template>
	<NewModal ref="modal" no-padding scrollable max-width="560px" width="560px">
		<template #title>
			<span class="text-2xl font-semibold text-contrast">
				{{ formatMessage(messages.header) }}
			</span>
		</template>

		<div class="flex flex-col gap-3 p-6">
			<div class="flex flex-wrap items-center gap-2">
				<StyledInput
					v-model="query"
					:icon="SearchIcon"
					type="text"
					autocomplete="off"
					:spellcheck="false"
					clearable
					:placeholder="formatMessage(messages.searchPlaceholder)"
					wrapper-class="w-full flex-grow sm:w-auto"
					@keydown.enter="showOpen && openOtherCollection()"
				/>
				<ButtonStyled v-if="showOpen">
					<button :disabled="loadingOtherCollection" @click="openOtherCollection">
						<SpinnerIcon v-if="loadingOtherCollection" class="animate-spin" aria-hidden="true" />
						<ExternalIcon v-else aria-hidden="true" />
						{{ formatMessage(messages.openButton) }}
					</button>
				</ButtonStyled>
			</div>

			<div class="h-px bg-divider" />

			<div v-if="!userId" class="flex flex-col items-center gap-3 py-6 text-secondary">
				{{ formatMessage(messages.signInPrompt) }}
				<ButtonStyled color="brand">
					<button @click="auth.requestSignIn(route.fullPath)">
						<LogInIcon aria-hidden="true" />
						{{ formatMessage(messages.signInButton) }}
					</button>
				</ButtonStyled>
			</div>
			<div v-else-if="isPending" class="flex items-center justify-center py-6">
				<LoadingIndicator />
			</div>
			<div
				v-else-if="!showFollowing && filteredCollections.length === 0"
				class="flex items-center justify-center py-6 text-secondary"
			>
				{{ formatMessage(query.trim() ? messages.noMatches : messages.noCollections) }}
			</div>
			<div v-else class="flex max-h-[400px] flex-col gap-0.5 overflow-y-auto">
				<button
					v-if="showFollowing"
					class="flex cursor-pointer items-center gap-2.5 rounded-xl border-0 bg-transparent px-2 py-2 text-left hover:bg-surface-3"
					:disabled="loadingCollectionId !== null"
					@click="pickFollowing"
				>
					<Avatar src="https://cdn.modrinth.com/follow-collection.png" size="2.5rem" />
					<div class="flex min-w-0 flex-1 flex-col">
						<span class="truncate font-semibold text-contrast">{{ followingLabel }}</span>
						<span class="text-sm text-secondary">
							{{ formatMessage(messages.followingHint) }}
						</span>
					</div>
					<SpinnerIcon
						v-if="loadingCollectionId === 'following'"
						class="size-5 animate-spin text-secondary"
						aria-hidden="true"
					/>
					<HeartIcon v-else class="size-5 text-secondary" aria-hidden="true" />
				</button>
				<button
					v-for="collection in filteredCollections"
					:key="collection.id"
					class="flex cursor-pointer items-center gap-2.5 rounded-xl border-0 bg-transparent px-2 py-2 text-left hover:bg-surface-3"
					:disabled="loadingCollectionId !== null"
					@click="pickCollection(collection)"
				>
					<Avatar :src="collection.icon_url" size="2.5rem" />
					<div class="flex min-w-0 flex-1 flex-col">
						<span class="truncate font-semibold text-contrast">{{ collection.name }}</span>
						<span class="text-sm text-secondary">
							{{ formatMessage(messages.projectsCount, { count: collection.projects.length }) }}
						</span>
					</div>
					<SpinnerIcon
						v-if="loadingCollectionId === collection.id"
						class="size-5 animate-spin text-secondary"
						aria-hidden="true"
					/>
					<ChevronRightIcon v-else class="size-5 text-secondary" aria-hidden="true" />
				</button>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	ChevronRightIcon,
	ExternalIcon,
	HeartIcon,
	LogInIcon,
	SearchIcon,
	SpinnerIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	commonMessages,
	defineMessages,
	injectAuth,
	injectModrinthClient,
	injectNotificationManager,
	LoadingIndicator,
	NewModal,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'

import { get_project_many } from '@/helpers/cache.js'
import { isCollectionLink, parseCollectionId } from '@/helpers/collections'

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const auth = injectAuth()
const client = injectModrinthClient()
const route = useRoute()

const messages = defineMessages({
	header: {
		id: 'app.collection-picker.header',
		defaultMessage: 'Add from collection',
	},
	signInPrompt: {
		id: 'app.collection-picker.sign-in-prompt',
		defaultMessage: 'Sign in to view your collections',
	},
	signInButton: {
		id: 'app.collection-picker.sign-in-button',
		defaultMessage: 'Sign in',
	},
	noCollections: {
		id: 'app.collection-picker.empty',
		defaultMessage: "You don't have any collections yet",
	},
	projectsCount: {
		id: 'app.collection-picker.projects-count',
		defaultMessage: '{count, plural, =0 {No projects} one {# project} other {# projects}}',
	},
	searchPlaceholder: {
		id: 'app.collection-picker.search-placeholder',
		defaultMessage: 'Search your collections, or paste a link...',
	},
	noMatches: {
		id: 'app.collection-picker.no-matches',
		defaultMessage: 'No collections match your search',
	},
	followingHint: {
		id: 'app.collection-picker.following-hint',
		defaultMessage: 'Projects you follow',
	},
	openButton: {
		id: 'app.collection-picker.open-button',
		defaultMessage: 'Open',
	},
	invalidCollectionLink: {
		id: 'app.collection-picker.invalid-collection-link',
		defaultMessage: 'Invalid collection link',
	},
	invalidCollectionLinkText: {
		id: 'app.collection-picker.invalid-collection-link-text',
		defaultMessage: 'Paste a modrinth.com collection URL or a collection ID.',
	},
})

const emit = defineEmits<{
	select: [collection: Labrinth.Collections.Collection, projects: Labrinth.Projects.v2.Project[]]
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const isOpen = ref(false)
const loadingCollectionId = ref<string | null>(null)
const query = ref('')
const loadingOtherCollection = ref(false)
const showOpen = computed(() => isCollectionLink(query.value))

const userId = computed(() => auth.user.value?.id)

const { data: collections, isPending } = useQuery({
	queryKey: computed(() => ['user', userId.value, 'collections']),
	queryFn: () => client.labrinth.users_v2.getCollections(userId.value!),
	enabled: computed(() => !!userId.value && isOpen.value),
})

const filteredCollections = computed(() => {
	const q = query.value.trim().toLowerCase()
	if (!q) return collections.value ?? []
	return (collections.value ?? []).filter((c) => c.name.toLowerCase().includes(q))
})

const followingLabel = computed(() => formatMessage(commonMessages.followedProjectsLabel))
const showFollowing = computed(() => {
	const q = query.value.trim().toLowerCase()
	return !!userId.value && (!q || followingLabel.value.toLowerCase().includes(q))
})

async function pickCollection(collection: Labrinth.Collections.Collection) {
	loadingCollectionId.value = collection.id
	try {
		const fetched: Labrinth.Projects.v2.Project[] =
			collection.projects.length > 0 ? await get_project_many(collection.projects) : []
		const projects = (fetched ?? []).filter((project) => !!project)
		for (const project of projects) {
			project.categories = (project.categories ?? []).concat(project.loaders ?? [])
		}
		emit('select', collection, projects)
		modal.value?.hide()
	} catch (err) {
		handleError(err as Error)
	} finally {
		loadingCollectionId.value = null
	}
}

async function pickFollowing() {
	loadingCollectionId.value = 'following'
	try {
		const fetched = await client.labrinth.users_v2.getFollowedProjects(userId.value!)
		const projects = (fetched ?? []).filter((project) => !!project)
		for (const project of projects) {
			project.categories = (project.categories ?? []).concat(project.loaders ?? [])
		}
		emit(
			'select',
			{ id: 'following', name: followingLabel.value } as Labrinth.Collections.Collection,
			projects,
		)
		modal.value?.hide()
	} catch (err) {
		handleError(err as Error)
	} finally {
		loadingCollectionId.value = null
	}
}

async function openOtherCollection() {
	const id = parseCollectionId(query.value)
	if (!id) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.invalidCollectionLink),
			text: formatMessage(messages.invalidCollectionLinkText),
		})
		return
	}

	loadingOtherCollection.value = true
	try {
		const collection = await client.labrinth.collections.get(id)
		await pickCollection(collection)
		query.value = ''
	} catch (err) {
		handleError(err as Error)
	} finally {
		loadingOtherCollection.value = false
	}
}

function show() {
	isOpen.value = true
	loadingCollectionId.value = null
	query.value = ''
	loadingOtherCollection.value = false
	modal.value?.show()
}

defineExpose({ show })
</script>
