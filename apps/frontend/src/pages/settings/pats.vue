<template>
	<div class="flex flex-col">
		<ConfirmModal
			ref="modal_confirm"
			:title="formatMessage(deleteModalMessages.title)"
			:description="formatMessage(deleteModalMessages.description)"
			:proceed-label="formatMessage(deleteModalMessages.action)"
			@proceed="removePat(deletePatIndex)"
		/>
		<NewModal
			ref="patModal"
			:header="
				editPatId !== null
					? formatMessage(createModalMessages.editTitle)
					: formatMessage(createModalMessages.createTitle)
			"
			:width="'550px'"
		>
			<div class="flex flex-col gap-6">
				<div class="flex w-full flex-col gap-2.5">
					<label for="pat-name">
						<span class="font-semibold">{{ formatMessage(createModalMessages.nameLabel) }}</span>
					</label>
					<Input
						id="pat-name"
						v-model="name"
						:maxlength="2048"
						:placeholder="formatMessage(createModalMessages.namePlaceholder)"
					/>
				</div>

				<div class="flex w-full flex-col gap-2.5">
					<label for="pat-scopes">
						<span class="font-semibold">{{ formatMessage(commonMessages.scopesLabel) }}</span>
					</label>
					<div
						id="pat-scopes"
						class="scope-items grid grid-cols-1 gap-x-6 gap-y-4 min-[600px]:grid-cols-2"
					>
						<div
							v-for="category in scopeCategories"
							:key="category.name"
							class="flex flex-col gap-1.5"
						>
							<h4 class="m-0 border-b border-divider text-base font-medium text-primary">
								{{ category.name }}
							</h4>
							<div class="flex flex-col gap-2">
								<Checkbox
									v-for="scope in category.scopes"
									:key="scope"
									:label="scopesToLabels(getScopeValue(scope)).join(', ')"
									:model-value="hasScope(scopesVal, scope)"
									@update:model-value="scopesVal = toggleScope(scopesVal, scope)"
								/>
							</div>
						</div>
					</div>
				</div>

				<div class="flex w-full flex-col gap-2.5">
					<label for="pat-expires">
						<span class="font-semibold">{{ formatMessage(createModalMessages.expiresLabel) }}</span>
					</label>
					<DatePicker
						id="pat-expires"
						v-model="expires"
						:min-date="minimumPatExpiry"
						show-today
						wrapper-class="w-full"
					/>
				</div>

				<div class="ml-auto mt-4 flex gap-2">
					<Button type="outlined" @click="patModal.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<Button
						v-if="editPatId !== null"
						type="colored"
						color="brand"
						:disabled="loading || !name || !isExpiryInFuture"
						@click="editPat"
					>
						<SaveIcon />
						{{ formatMessage(commonMessages.saveChangesButton) }}
					</Button>
					<Button
						v-else
						type="colored"
						color="brand"
						:disabled="loading || !name || !isExpiryInFuture"
						@click="createPat"
					>
						<PlusIcon />
						{{ formatMessage(createModalMessages.action) }}
					</Button>
				</div>
			</div>
		</NewModal>

		<div class="flex flex-wrap items-center justify-between gap-3">
			<h2 class="m-0 text-2xl font-semibold">{{ formatMessage(commonSettingsMessages.pats) }}</h2>
			<Button type="colored" color="brand" @click="openCreatePat">
				<PlusIcon />
				{{ formatMessage(messages.create) }}
			</Button>
		</div>
		<p class="mb-4 mt-2 leading-tight">
			<IntlFormatted :message-id="messages.description">
				<template #doc-link="{ children }">
					<a class="text-link" href="https://docs.modrinth.com">
						<component :is="() => children" />
					</a>
				</template>
			</IntlFormatted>
		</p>

		<template v-if="isLoading">
			<div class="flex flex-col gap-3">
				<div
					v-for="i in 2"
					:key="`pat-skeleton-${i}`"
					class="overflow-hidden rounded-2xl border border-solid border-surface-4 bg-surface-3"
				>
					<div
						class="flex flex-wrap items-center gap-3 border-0 border-b border-solid border-surface-4 p-4"
					>
						<div class="size-12 shrink-0 animate-pulse rounded-xl bg-surface-4" />
						<div class="flex min-w-0 flex-1 flex-col gap-2">
							<div class="h-4 w-36 animate-pulse rounded-lg bg-surface-4" />
							<div class="h-3 w-64 max-w-full animate-pulse rounded-lg bg-surface-4" />
						</div>
						<div class="mb-auto flex gap-2">
							<div class="h-9 w-28 animate-pulse rounded-xl bg-surface-4" />
							<div class="h-9 w-40 animate-pulse rounded-xl bg-surface-4" />
						</div>
					</div>
					<div class="bg-surface-2 p-4">
						<div class="mb-3 h-3 w-28 animate-pulse rounded-lg bg-surface-3" />
						<div class="grid gap-2 sm:grid-cols-2">
							<div
								v-for="j in 4"
								:key="`pat-skeleton-${i}-scope-${j}`"
								class="h-4 w-full animate-pulse rounded-lg bg-surface-3"
							/>
						</div>
					</div>
				</div>
			</div>
		</template>
		<EmptyState
			v-else-if="displayPats.length === 0"
			type="empty"
			:heading="formatMessage(messages.emptyStateHeading)"
			:description="formatMessage(messages.emptyStateDescription)"
		/>
		<div v-else class="flex flex-col gap-3">
			<div
				v-for="pat in displayPats"
				:key="pat.id"
				class="rounded-2xl border border-solid border-surface-4 bg-surface-3"
			>
				<div
					class="flex flex-wrap items-start justify-between gap-3 border-0 border-b border-solid border-surface-4 p-4"
				>
					<div class="flex min-w-0 flex-1 items-center gap-3">
						<div
							class="flex size-12 shrink-0 items-center justify-center rounded-xl border border-solid border-surface-5 bg-surface-4 text-contrast"
						>
							<KeyIcon class="size-6" />
						</div>
						<div class="flex min-w-0 flex-col gap-0.5">
							<h2 class="m-0 break-words text-base font-semibold">{{ pat.name }}</h2>
							<PageHeaderMetadata>
								<PageHeaderMetadataItem
									v-if="pat.last_used"
									:tooltip="formatDateTime(pat.last_used)"
								>
									{{
										formatMessage(tokenMessages.lastUsed, {
											ago: formatRelativeTime(pat.last_used),
										})
									}}
								</PageHeaderMetadataItem>
								<PageHeaderMetadataItem v-else>
									{{ formatMessage(tokenMessages.neverUsed) }}
								</PageHeaderMetadataItem>
								<PageHeaderMetadataItem
									:class="isPatExpired(pat) ? '!text-red' : undefined"
									:tooltip="Date.parse(pat.expires) ? formatDateTime(pat.expires) : undefined"
								>
									{{
										isPatExpired(pat)
											? formatMessage(tokenMessages.expiredAgo, {
													ago: formatRelativeTime(pat.expires),
												})
											: formatMessage(tokenMessages.expiresIn, {
													inTime: formatRelativeTime(pat.expires),
												})
									}}
								</PageHeaderMetadataItem>
								<PageHeaderMetadataItem :tooltip="formatDateTime(pat.created)">
									{{
										formatMessage(commonMessages.createdAgoLabel, {
											ago: formatRelativeTime(pat.created),
										})
									}}
								</PageHeaderMetadataItem>
							</PageHeaderMetadata>
						</div>
					</div>
					<div class="flex shrink-0 flex-wrap gap-2">
						<Button @click="openEditPat(pat)">
							<EditIcon />
							{{ formatMessage(tokenMessages.edit) }}
						</Button>
						<Button type="colored" color="red" @click="confirmRemovePat(pat.id)">
							<XCircleIcon />
							{{ formatMessage(tokenMessages.revoke) }}
						</Button>
					</div>
				</div>
				<div class="flex flex-col gap-3 rounded-b-2xl bg-surface-2 p-4">
					<div v-if="visibleToken(pat)">
						<label :for="`pat-token-${pat.id}`" class="mb-1 flex font-semibold text-contrast">
							{{ formatMessage(tokenMessages.accessToken) }}
						</label>
						<div :id="`pat-token-${pat.id}`" class="max-w-full overflow-x-auto">
							<CopyCode :text="visibleToken(pat)" />
						</div>
						<p v-if="isNewlyCreatedToken(pat)" class="mb-0 mt-2 text-sm text-orange">
							{{ formatMessage(tokenMessages.tokenDisclaimer) }}
						</p>
					</div>
					<div>
						<label :for="`pat-permissions-${pat.id}`" class="mb-1 flex font-semibold text-contrast">
							{{ formatMessage(commonMessages.permissionsLabel) }}
						</label>
						<p
							v-if="patScopes(pat).length === 0"
							:id="`pat-permissions-${pat.id}`"
							class="m-0 text-secondary"
						>
							{{ formatMessage(tokenMessages.noPermissions) }}
						</p>
						<div v-else :id="`pat-permissions-${pat.id}`" class="grid gap-1 sm:grid-cols-2">
							<div
								v-for="(scope, index) in patScopes(pat)"
								:key="`${pat.id}-scope-${index}`"
								class="flex items-center gap-1"
							>
								<CheckIcon class="size-5 shrink-0 text-green" />
								{{ scope }}
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>
<script setup>
import {
	CheckIcon,
	EditIcon,
	KeyIcon,
	PlusIcon,
	SaveIcon,
	XCircleIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Button,
	Checkbox,
	commonMessages,
	commonSettingsMessages,
	ConfirmModal,
	CopyCode,
	DatePicker,
	defineMessages,
	EmptyState,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	IntlFormatted,
	NewModal,
	PageHeaderMetadata,
	PageHeaderMetadataItem,
	useFormatDateTime,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'

import {
	getScopeValue,
	hasScope,
	scopeCategoryMessages,
	scopeList,
	toggleScope,
	useScopes,
} from '~/composables/auth/scopes.ts'

const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const formatRelativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})

const createModalMessages = defineMessages({
	createTitle: {
		id: 'settings.pats.modal.create.title',
		defaultMessage: 'Create personal access token',
	},
	editTitle: {
		id: 'settings.pats.modal.edit.title',
		defaultMessage: 'Edit personal access token',
	},
	nameLabel: {
		id: 'settings.pats.modal.create.name.label',
		defaultMessage: 'Name',
	},
	namePlaceholder: {
		id: 'settings.pats.modal.create.name.placeholder',
		defaultMessage: "Enter the PAT's name...",
	},
	expiresLabel: {
		id: 'settings.pats.modal.create.expires.label',
		defaultMessage: 'Expires',
	},
	action: {
		id: 'settings.pats.modal.create.action',
		defaultMessage: 'Create PAT',
	},
})

const deleteModalMessages = defineMessages({
	title: {
		id: 'settings.pats.modal.revoke.title',
		defaultMessage: 'Are you sure you want to revoke this token?',
	},
	description: {
		id: 'settings.pats.modal.revoke.description',
		defaultMessage:
			'This token will immediately be invalidated and anything you have set up using it will stop working. This action cannot be undone.',
	},
	action: {
		id: 'settings.pats.modal.revoke.action',
		defaultMessage: 'Revoke this token',
	},
})

const messages = defineMessages({
	description: {
		id: 'settings.pats.description',
		defaultMessage:
			"PATs can be used to access Modrinth's API. They can be created and revoked at any time. For more information, see <doc-link>Modrinth's API documentation</doc-link>.",
	},
	create: {
		id: 'settings.pats.action.create',
		defaultMessage: 'Create a PAT',
	},
	emptyStateHeading: {
		id: 'settings.pats.empty-state.heading',
		defaultMessage: 'No personal access tokens',
	},
	emptyStateDescription: {
		id: 'settings.pats.empty-state.description',
		defaultMessage: 'Tokens you create will appear here.',
	},
})

const tokenMessages = defineMessages({
	edit: {
		id: 'settings.pats.token.action.edit',
		defaultMessage: 'Edit token',
	},
	revoke: {
		id: 'settings.pats.token.action.revoke',
		defaultMessage: 'Revoke token',
	},
	lastUsed: {
		id: 'settings.pats.token.last-used',
		defaultMessage: 'Last used {ago}',
	},
	neverUsed: {
		id: 'settings.pats.token.never-used',
		defaultMessage: 'Never used',
	},
	expiresIn: {
		id: 'settings.pats.token.expires-in',
		defaultMessage: 'Expires {inTime}',
	},
	expiredAgo: {
		id: 'settings.pats.token.expired-ago',
		defaultMessage: 'Expired {ago}',
	},
	accessToken: {
		id: 'settings.pats.token.access-token',
		defaultMessage: 'Access token',
	},
	tokenDisclaimer: {
		id: 'settings.pats.token.disclaimer',
		defaultMessage: `Save this token now. You will only be able to view it once. When you leave this page, you won't be able to see it again.`,
	},
	noPermissions: {
		id: 'settings.pats.token.no-permissions',
		defaultMessage: 'No permissions',
	},
})

definePageMeta({
	middleware: 'auth',
})

useHead({
	title: () => `${formatMessage(commonSettingsMessages.pats)} - Modrinth`,
})

const data = useNuxtApp()
const { scopesToLabels, scopesToDefinitions } = useScopes()
const patModal = ref()
const minimumPatExpiry = data.$dayjs().add(1, 'day').format('YYYY-MM-DD')

const editPatId = ref(null)

const name = ref(null)
const scopesVal = ref(BigInt(0))
const expires = ref(null)
const createdPatTokens = ref({})
const isExpiryInFuture = computed(
	() => expires.value && data.$dayjs(expires.value).isAfter(data.$dayjs(), 'day'),
)

const deletePatIndex = ref(null)
const modal_confirm = ref()

const loading = ref(false)

const { data: pats, isLoading } = useQuery({
	queryKey: ['pat'],
	queryFn: () => client.labrinth.pats_v2.list(),
})

const displayPats = computed(() => {
	return [...(pats.value ?? [])].sort(
		(a, b) => new Date(b.created).getTime() - new Date(a.created).getTime(),
	)
})

const scopeCategories = computed(() => {
	return [
		{
			name: formatMessage(scopeCategoryMessages.categoryUserAccount),
			scopes: scopeList.filter((s) => s.startsWith('USER_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categoryProjects),
			scopes: scopeList.filter((s) => s.startsWith('PROJECT_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categoryVersions),
			scopes: scopeList.filter((s) => s.startsWith('VERSION_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categoryCollections),
			scopes: scopeList.filter((s) => s.startsWith('COLLECTION_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categoryOrganizations),
			scopes: scopeList.filter((s) => s.startsWith('ORGANIZATION_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categoryReports),
			scopes: scopeList.filter((s) => s.startsWith('REPORT_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categoryThreads),
			scopes: scopeList.filter((s) => s.startsWith('THREAD_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categoryPats),
			scopes: scopeList.filter((s) => s.startsWith('PAT_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categorySessions),
			scopes: scopeList.filter((s) => s.startsWith('SESSION_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categoryNotifications),
			scopes: scopeList.filter((s) => s.startsWith('NOTIFICATION_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categoryPayouts),
			scopes: scopeList.filter((s) => s.startsWith('PAYOUTS_')),
		},
		{
			name: formatMessage(scopeCategoryMessages.categoryAnalytics),
			scopes: scopeList.filter(
				(s) => s.startsWith('ANALYTICS') || s.startsWith('PERFORM_ANALYTICS'),
			),
		},
	].filter((c) => c.scopes.length > 0)
})

function openCreatePat() {
	name.value = null
	scopesVal.value = BigInt(0)
	expires.value = null
	editPatId.value = null
	patModal.value.show()
}

function openEditPat(pat) {
	editPatId.value = pat.id
	name.value = pat.name
	scopesVal.value = BigInt(pat.scopes)
	expires.value = data.$dayjs(pat.expires).format('YYYY-MM-DD')
	patModal.value.show()
}

function confirmRemovePat(id) {
	deletePatIndex.value = id
	modal_confirm.value.show()
}

function visibleToken(pat) {
	return createdPatTokens.value[pat.id] || pat.access_token || null
}

function isNewlyCreatedToken(pat) {
	return Boolean(createdPatTokens.value[pat.id])
}

function isPatExpired(pat) {
	return new Date(pat.expires) <= new Date()
}

function patScopes(pat) {
	return scopesToDefinitions(BigInt(pat.scopes || 0))
}

async function createPat() {
	startLoading()
	loading.value = true
	try {
		const res = await client.labrinth.pats_v2.create({
			name: name.value,
			scopes: Number(scopesVal.value),
			expires: data.$dayjs(expires.value).toISOString(),
		})
		if (res.access_token) {
			createdPatTokens.value[res.id] = res.access_token
		}
		queryClient.setQueryData(['pat'], (old) => [...(old || []), res])
		patModal.value.hide()
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	loading.value = false
	stopLoading()
}

async function editPat() {
	startLoading()
	loading.value = true
	try {
		await client.labrinth.pats_v2.modify(editPatId.value, {
			name: name.value,
			scopes: Number(scopesVal.value),
			expires: data.$dayjs(expires.value).toISOString(),
		})
		await queryClient.invalidateQueries({ queryKey: ['pat'] })
		patModal.value.hide()
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	loading.value = false
	stopLoading()
}

async function removePat(id) {
	startLoading()
	const previous = queryClient.getQueryData(['pat'])
	queryClient.setQueryData(['pat'], (old) => (old ?? []).filter((pat) => pat.id !== id))
	try {
		await client.labrinth.pats_v2.delete(id)
		createdPatTokens.value = Object.fromEntries(
			Object.entries(createdPatTokens.value).filter(([key]) => key !== String(id)),
		)
	} catch (err) {
		queryClient.setQueryData(['pat'], previous)
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
}
</script>
<style lang="scss" scoped>
.scope-items :deep(.checkbox-outer) {
	justify-content: flex-start !important;
}
</style>
