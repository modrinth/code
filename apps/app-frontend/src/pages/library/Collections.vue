<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ExternalIcon, GlobeIcon, LinkIcon, LockIcon, LogInIcon } from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
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

import { parseCollectionId } from '@/helpers/collections'

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
	noCollections: {
		id: 'app.collections.empty',
		defaultMessage: "You don't have any collections yet",
	},
	noCollectionsHint: {
		id: 'app.collections.empty-hint',
		defaultMessage: 'Create collections on modrinth.com and view them here',
	},
	projectsCount: {
		id: 'app.collections.projects-count',
		defaultMessage: '{count, plural, =0 {No projects} one {# project} other {# projects}}',
	},
	openCollectionPlaceholder: {
		id: 'app.collections.open-collection-placeholder',
		defaultMessage: 'Paste a collection link or ID to open any collection...',
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

const openCollectionInput = ref('')

function openCollection() {
	const id = parseCollectionId(openCollectionInput.value)
	if (!id) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.invalidCollectionLink),
			text: formatMessage(messages.invalidCollectionLinkText),
		})
		return
	}
	openCollectionInput.value = ''
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
				v-model="openCollectionInput"
				:icon="LinkIcon"
				type="text"
				autocomplete="off"
				:spellcheck="false"
				:placeholder="formatMessage(messages.openCollectionPlaceholder)"
				wrapper-class="w-full flex-grow sm:w-auto"
				@keydown.enter="openCollection"
			/>
			<ButtonStyled>
				<button :disabled="!openCollectionInput.trim()" @click="openCollection">
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
		<div
			v-else-if="collections && collections.length > 0"
			class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3"
		>
			<router-link
				v-for="collection in collections"
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
		<EmptyState v-else type="empty-inbox">
			<template #heading>{{ formatMessage(messages.noCollections) }}</template>
			<template #description>{{ formatMessage(messages.noCollectionsHint) }}</template>
		</EmptyState>
	</div>
</template>
