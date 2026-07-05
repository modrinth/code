<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	ExternalIcon,
	GlobeIcon,
	HeartIcon,
	LinkIcon,
	LockIcon,
	LogInIcon,
	SearchIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	commonMessages,
	defineMessages,
	EmptyState,
	injectAuth,
	injectModrinthClient,
	injectNotificationManager,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import { isCollectionLink, parseCollectionId } from '@/helpers/collections'

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const auth = injectAuth()
const client = injectModrinthClient()
const router = useRouter()

const messages = defineMessages({
	signInPrompt: {
		id: 'app.collections.sign-in-prompt',
		defaultMessage: 'Sign in to view your collections',
	},
	signInButton: {
		id: 'app.collections.sign-in-button',
		defaultMessage: 'Sign in',
	},
	followingCardHint: {
		id: 'app.collections.following-card-hint',
		defaultMessage: 'Projects you follow',
	},
	projectsCount: {
		id: 'app.collections.projects-count',
		defaultMessage: '{count, plural, =0 {No projects} one {# project} other {# projects}}',
	},
	searchPlaceholder: {
		id: 'app.collections.search-placeholder',
		defaultMessage: 'Search your collections, or paste a link to open another...',
	},
	openCollectionButton: {
		id: 'app.collections.open-collection-button',
		defaultMessage: 'Open',
	},
	invalidCollectionLink: {
		id: 'app.collections.invalid-collection-link',
		defaultMessage: 'Invalid collection link',
	},
	invalidCollectionLinkText: {
		id: 'app.collections.invalid-collection-link-text',
		defaultMessage: 'Paste a modrinth.com collection URL or a collection ID.',
	},
})

const userId = computed(() => auth.user.value?.id)

const { data: collections, isPending } = useQuery({
	queryKey: computed(() => ['user', userId.value, 'collections']),
	queryFn: () => client.labrinth.users_v2.getCollections(userId.value!),
	enabled: computed(() => !!userId.value),
})

const query = ref('')
const showOpen = computed(() => isCollectionLink(query.value))

const filteredCollections = computed(() => {
	const q = query.value.trim().toLowerCase()
	if (!q) return collections.value ?? []
	return (collections.value ?? []).filter((c) => c.name.toLowerCase().includes(q))
})

const showFollowingCard = computed(() => {
	const q = query.value.trim().toLowerCase()
	return (
		!q || formatMessage(commonMessages.followedProjectsLabel).toLowerCase().includes(q)
	)
})

function openCollection() {
	const id = parseCollectionId(query.value)
	if (!id) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.invalidCollectionLink),
			text: formatMessage(messages.invalidCollectionLinkText),
		})
		return
	}
	query.value = ''
	router.push(`/collection/${id}`)
}

function statusIcon(collection: Labrinth.Collections.Collection) {
	switch (collection.status) {
		case 'listed':
			return GlobeIcon
		case 'unlisted':
			return LinkIcon
		default:
			return LockIcon
	}
}
</script>

<template>
	<div class="flex flex-col gap-3">
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
				@keydown.enter="showOpen && openCollection()"
			/>
			<ButtonStyled v-if="showOpen">
				<button @click="openCollection">
					<ExternalIcon aria-hidden="true" />
					{{ formatMessage(messages.openCollectionButton) }}
				</button>
			</ButtonStyled>
		</div>

		<EmptyState v-if="!userId" type="empty-inbox">
			<template #heading>{{ formatMessage(messages.signInPrompt) }}</template>
			<template #actions>
				<ButtonStyled color="brand">
					<button @click="auth.requestSignIn('/library/collections')">
						<LogInIcon />
						{{ formatMessage(messages.signInButton) }}
					</button>
				</ButtonStyled>
			</template>
		</EmptyState>
		<div v-else-if="isPending" class="flex flex-col gap-3">
			<div v-for="i in 3" :key="i" class="h-20 animate-pulse rounded-2xl bg-surface-3" />
		</div>
		<div v-else class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
			<router-link
				v-if="showFollowingCard"
				to="/collection/following"
				class="flex items-center gap-3 rounded-2xl bg-surface-3 p-4 text-primary transition-all hover:bg-surface-4 hover:brightness-105 active:scale-[0.98]"
			>
				<Avatar src="https://cdn.modrinth.com/follow-collection.png" size="48px" />
				<div class="flex min-w-0 flex-col gap-1">
					<span class="truncate font-semibold text-contrast">
						{{ formatMessage(commonMessages.followedProjectsLabel) }}
					</span>
					<span class="flex items-center gap-1.5 text-sm text-secondary">
						<HeartIcon class="size-4 shrink-0" aria-hidden="true" />
						{{ formatMessage(messages.followingCardHint) }}
					</span>
				</div>
			</router-link>
			<router-link
				v-for="collection in filteredCollections"
				:key="collection.id"
				:to="`/collection/${collection.id}`"
				class="flex items-center gap-3 rounded-2xl bg-surface-3 p-4 text-primary transition-all hover:bg-surface-4 hover:brightness-105 active:scale-[0.98]"
			>
				<Avatar :src="collection.icon_url" size="48px" />
				<div class="flex min-w-0 flex-col gap-1">
					<span class="truncate font-semibold text-contrast">{{ collection.name }}</span>
					<span class="flex items-center gap-1.5 text-sm text-secondary">
						<component :is="statusIcon(collection)" class="size-4 shrink-0" aria-hidden="true" />
						{{ formatMessage(messages.projectsCount, { count: collection.projects.length }) }}
					</span>
				</div>
			</router-link>
		</div>
	</div>
</template>
