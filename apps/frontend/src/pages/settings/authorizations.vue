<template>
	<div class="flex flex-col">
		<ConfirmModal
			ref="modal_confirm"
			:title="formatMessage(messages.revokeConfirmTitle)"
			:description="formatMessage(messages.revokeConfirmDescription)"
			:proceed-label="formatMessage(messages.revokeAction)"
			@proceed="revokeApp(revokingId)"
		/>
		<h2 class="m-0 text-2xl font-semibold">
			{{ formatMessage(commonSettingsMessages.authorizedApps) }}
		</h2>
		<p class="mb-4 mt-2">
			{{ formatMessage(messages.description) }}
		</p>
		<template v-if="isLoading">
			<div class="flex flex-col gap-4">
				<div
					v-for="i in 2"
					:key="`authorization-skeleton-${i}`"
					class="overflow-hidden rounded-2xl border border-solid border-surface-4 bg-surface-3"
				>
					<div class="flex items-center gap-3 border-0 border-b border-solid border-surface-4 p-4">
						<div class="size-12 shrink-0 animate-pulse rounded-xl bg-surface-4" />
						<div class="flex flex-1 flex-col gap-2">
							<div class="h-4 w-36 animate-pulse rounded-lg bg-surface-4" />
							<div class="h-3 w-48 animate-pulse rounded-lg bg-surface-4" />
						</div>
						<div class="h-9 w-24 animate-pulse rounded-xl bg-surface-4" />
					</div>
					<div class="bg-surface-2 p-4">
						<div class="mb-3 h-3 w-28 animate-pulse rounded-lg bg-surface-3" />
						<div class="grid gap-2 sm:grid-cols-2">
							<div
								v-for="j in 4"
								:key="`authorization-skeleton-${i}-scope-${j}`"
								class="h-4 w-full animate-pulse rounded-lg bg-surface-3"
							/>
						</div>
					</div>
				</div>
			</div>
		</template>
		<EmptyState
			v-else-if="showEmptyState"
			type="done"
			:heading="formatMessage(messages.emptyStateHeading)"
			:description="formatMessage(messages.emptyStateDescription)"
		/>
		<div v-else class="flex flex-col gap-4">
			<AuthorizationCard
				v-for="authorization in appInfoLookup"
				:key="authorization.id"
				:authorization="authorization"
				@revoke="onRevoke"
			/>
		</div>
	</div>
</template>
<script setup>
import {
	commonMessages,
	commonSettingsMessages,
	ConfirmModal,
	defineMessages,
	EmptyState,
	injectModrinthClient,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'

import AuthorizationCard from '~/components/ui/AuthorizationCard.vue'

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	headTitle: {
		id: 'settings.authorizations.head-title',
		defaultMessage: 'Authorizations',
	},
	description: {
		id: 'settings.authorizations.description',
		defaultMessage:
			'When you authorize an application with your Modrinth account, you grant it access to your account. You can manage and review access to your account here at any time.',
	},
	emptyStateHeading: {
		id: 'settings.authorizations.empty-state.heading',
		defaultMessage: 'No authorized apps',
	},
	emptyStateDescription: {
		id: 'settings.authorizations.empty-state.description',
		defaultMessage: 'Applications you authorize will appear here.',
	},
	revokeConfirmTitle: {
		id: 'settings.authorizations.revoke.confirm.title',
		defaultMessage: 'Are you sure you want to revoke this application?',
	},
	revokeConfirmDescription: {
		id: 'settings.authorizations.revoke.confirm.description',
		defaultMessage:
			"This will revoke the application's access to your account. You can always re-authorize it later.",
	},
	revokeAction: {
		id: 'settings.authorizations.revoke.action',
		defaultMessage: 'Revoke',
	},
})

const revokingId = ref(null)
const modal_confirm = ref()

definePageMeta({
	middleware: 'auth',
})

useHead({
	title: () => `${formatMessage(messages.headTitle)} - Modrinth`,
})

const { data: usersApps, refetch: refresh } = useQuery({
	queryKey: ['oauth', 'authorizations'],
	queryFn: () => client.labrinth.oauth_internal.getAuthorizations(),
})

const { data: appInformation } = useQuery({
	queryKey: computed(() => ['oauth', 'apps', usersApps.value?.map((c) => c.app_id)]),
	queryFn: () => client.labrinth.oauth_internal.getApps(usersApps.value.map((c) => c.app_id)),
	enabled: computed(() => !!usersApps.value?.length),
})

const { data: appCreatorsInformation } = useQuery({
	queryKey: computed(() => ['users', appInformation.value?.map((c) => c.created_by)]),
	queryFn: () =>
		client.labrinth.users_v2.getMultiple(appInformation.value.map((c) => c.created_by)),
	enabled: computed(() => !!appInformation.value?.length),
})

const appInfoLookup = computed(() => {
	if (!usersApps.value || !appInformation.value) {
		return []
	}
	if (appInformation.value.length === 0) {
		return []
	}
	if (!appCreatorsInformation.value) {
		return []
	}

	return usersApps.value
		.map((app) => {
			const info = appInformation.value.find((c) => c.id === app.app_id)
			const owner = appCreatorsInformation.value.find((c) => c.id === info?.created_by)
			if (!info || !owner) {
				return null
			}
			return {
				...app,
				app: info,
				owner,
			}
		})
		.filter(Boolean)
})

const showEmptyState = computed(() => {
	if (!usersApps.value) {
		return false
	}
	if (usersApps.value.length === 0) {
		return true
	}
	if (!appInformation.value) {
		return false
	}
	if (appInformation.value.length === 0) {
		return true
	}
	if (!appCreatorsInformation.value) {
		return false
	}
	return appInfoLookup.value.length === 0
})

const isLoading = computed(() => !showEmptyState.value && appInfoLookup.value.length === 0)

function onRevoke(appId) {
	revokingId.value = appId
	modal_confirm.value.show()
}

async function revokeApp(id) {
	try {
		await client.labrinth.oauth_internal.revokeAuthorization(id)
		revokingId.value = null
		await refresh()
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
}
</script>
