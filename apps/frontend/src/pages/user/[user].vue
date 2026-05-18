<template>
	<div v-if="user">
		<ModalCreation ref="modal_creation" />
		<CollectionCreateModal ref="modal_collection_creation" />
		<NewModal ref="editRoleModal" header="Edit role">
			<div class="flex w-80 flex-col gap-4">
				<div class="flex flex-col gap-2">
					<Combobox v-model="selectedRole" :options="roleOptions" placeholder="Select a role" />
				</div>
				<div class="flex justify-end gap-2">
					<ButtonStyled>
						<button @click="cancelRoleEdit">
							<XIcon />
							{{ formatMessage(commonMessages.cancelButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button
							:disabled="!selectedRole || selectedRole === user.role || isSavingRole"
							@click="saveRoleEdit"
						>
							<template v-if="isSavingRole">
								<SpinnerIcon class="animate-spin" /> {{ formatMessage(messages.savingLabel) }}
							</template>
							<template v-else>
								<SaveIcon /> {{ formatMessage(commonMessages.saveChangesButton) }}
							</template>
						</button>
					</ButtonStyled>
				</div>
			</div>
		</NewModal>
		<NewModal v-if="auth.user && isStaff(auth.user)" ref="userDetailsModal" header="User details">
			<div class="flex flex-col gap-3">
				<div v-if="isAdmin(auth.user)" class="flex flex-col gap-1">
					<span class="text-lg font-bold text-primary">{{
						formatMessage(commonMessages.emailLabel)
					}}</span>
					<div>
						<span
							v-tooltip="
								user.email_verified
									? formatMessage(messages.emailVerifiedTooltip)
									: formatMessage(messages.emailNotVerifiedTooltip)
							"
							class="flex w-fit items-center gap-1"
						>
							<span>{{ user.email }}</span>
							<CheckIcon v-if="user.email_verified" class="h-4 w-4 text-brand" />
							<XIcon v-else class="h-4 w-4 text-red" />
						</span>
					</div>
				</div>

				<div v-if="!isAdmin(auth.user)" class="flex flex-col gap-1">
					<span class="text-lg font-bold text-primary">{{
						formatMessage(messages.emailVerifiedLabel)
					}}</span>
					<span class="flex w-fit items-center gap-1">
						<CheckIcon v-if="user.email_verified" class="h-4 w-4 text-brand" />
						<XIcon v-else class="h-4 w-4 text-red" />
						{{
							user.email_verified
								? formatMessage(commonMessages.yesLabel)
								: formatMessage(commonMessages.noLabel)
						}}
					</span>
				</div>

				<div v-if="isAdmin(auth.user)" class="flex flex-col gap-1">
					<span class="text-lg font-bold text-primary">{{
						formatMessage(messages.authProvidersLabel)
					}}</span>
					<span>{{ user.auth_providers.join(', ') }}</span>
				</div>

				<div v-if="isAdmin(auth.user)" class="flex flex-col gap-1">
					<span class="text-lg font-bold text-primary">{{
						formatMessage(messages.paymentMethodsLabel)
					}}</span>
					<span>
						<template v-if="user.payout_data?.paypal_address">
							Paypal ({{ user.payout_data.paypal_address }} - {{ user.payout_data.paypal_country }})
						</template>
						<template v-if="user.payout_data?.paypal_address && user.payout_data?.venmo_address">
							,
						</template>
						<template v-if="user.payout_data?.venmo_address">
							Venmo ({{ user.payout_data.venmo_address }})
						</template>
					</span>
				</div>

				<div class="flex flex-col gap-1">
					<span class="text-lg font-bold text-primary">{{
						formatMessage(messages.hasPasswordLabel)
					}}</span>
					<span>
						{{
							user.has_password
								? formatMessage(commonMessages.yesLabel)
								: formatMessage(commonMessages.noLabel)
						}}
					</span>
				</div>

				<div class="flex flex-col gap-1">
					<span class="text-lg font-bold text-primary">{{
						formatMessage(messages.hasTotpLabel)
					}}</span>
					<span>
						{{
							user.has_totp
								? formatMessage(commonMessages.yesLabel)
								: formatMessage(commonMessages.noLabel)
						}}
					</span>
				</div>
			</div>
		</NewModal>
		<div class="new-page sidebar" :class="{ 'alt-layout': cosmetics.leftContentLayout }">
			<div class="normal-page__header py-4">
				<PageHeader
					:header="user.username"
					:summary="isModrinthUser ? null : profileHeaderSummary"
					:leading="profileHeaderLeading"
					:badges="profileHeaderBadges"
					:metadata="profileHeaderMetadata"
					:actions="profileHeaderActions"
				>
					<template v-if="isModrinthUser" #summary>
						<IntlFormatted :message-id="messages.officialAccountBio">
							<template #support-link>
								<a
									href="https://support.modrinth.com"
									class="text-link"
									target="_blank"
									rel="noopener noreferrer"
								>
									https://support.modrinth.com
								</a>
							</template>
							<template #email>
								<a
									href="mailto:support@modrinth.com"
									class="text-link"
									target="_blank"
									rel="noopener noreferrer"
								>
									support@modrinth.com
								</a>
							</template>
						</IntlFormatted>
					</template>
				</PageHeader>
			</div>
			<div class="normal-page__content">
				<div v-if="navLinks.length > 2" class="mb-4 max-w-full overflow-x-auto">
					<NavTabs :links="navLinks" replace />
				</div>
				<div v-if="projects?.length > 0">
					<ProjectCardList
						v-if="route.params.projectType !== 'collections'"
						:layout="cosmetics.searchDisplayMode.user"
					>
						<ProjectCard
							v-for="project in (route.params.projectType !== undefined
								? projects.filter(
										(x) =>
											x.project_type ===
											route.params.projectType.substr(0, route.params.projectType.length - 1),
									)
								: projects
							)
								.slice()
								.sort((a, b) => b.downloads - a.downloads)"
							:key="project.id"
							:link="`/${project.project_type ?? 'project'}/${project.slug ? project.slug : project.id}`"
							:title="project.title"
							:icon-url="project.icon_url"
							:date-updated="project.updated"
							:downloads="project.downloads"
							:summary="project.description"
							:tags="[...project.categories]"
							:all-tags="[
								...project.categories,
								...project.loaders,
								...project.additional_categories,
							]"
							:followers="project.followers"
							:banner="project.gallery.find((element) => element.featured)?.url"
							:color="project.color ?? undefined"
							:environment="{
								clientSide: project.client_side,
								serverSide: project.server_side,
							}"
							:layout="
								cosmetics.searchDisplayMode.user === 'grid' ||
								cosmetics.searchDisplayMode.user === 'gallery'
									? 'grid'
									: 'list'
							"
							:status="project.status"
						/>
					</ProjectCardList>
				</div>
				<div
					v-else-if="
						(route.params.projectType && route.params.projectType !== 'collections') ||
						(!route.params.projectType && collections?.length === 0)
					"
					class="error"
				>
					<UpToDate class="icon" />
					<br />
					<span v-if="auth.user && auth.user.id === user.id" class="preserve-lines text">
						<IntlFormatted :message-id="messages.profileNoProjectsAuthLabel">
							<template #create-link="{ children }">
								<a class="link" @click.prevent="$refs.modal_creation.show()">
									<component :is="() => children" />
								</a>
							</template>
						</IntlFormatted>
					</span>
					<span v-else class="text">{{ formatMessage(messages.profileNoProjectsLabel) }}</span>
				</div>
				<div
					v-if="!route.params.projectType || route.params.projectType === 'collections'"
					class="collections-grid"
					:class="{ 'mt-3': projects?.length > 0 }"
				>
					<nuxt-link
						v-for="collection in sortedCollections"
						:key="collection.id"
						:to="`/collection/${collection.id}`"
						class="card collection-item"
					>
						<div class="collection">
							<Avatar :src="collection.icon_url" size="64px" />
							<div class="details">
								<h2 class="title">{{ collection.name }}</h2>
								<div class="stats">
									<LibraryIcon aria-hidden="true" />
									{{ formatMessage(messages.collectionLabel) }}
								</div>
							</div>
						</div>
						<div class="description">
							{{ collection.description }}
						</div>
						<div class="stat-bar">
							<div class="stats">
								<BoxIcon />
								{{
									formatMessage(messages.collectionProjectsCount, {
										count: collection.projects?.length || 0,
									})
								}}
							</div>
							<div class="stats">
								<template v-if="collection.status === 'listed'">
									<GlobeIcon />
									<span> {{ formatMessage(commonMessages.publicLabel) }} </span>
								</template>
								<template v-else-if="collection.status === 'unlisted'">
									<LinkIcon />
									<span> {{ formatMessage(commonMessages.unlistedLabel) }} </span>
								</template>
								<template v-else-if="collection.status === 'private'">
									<LockIcon />
									<span> {{ formatMessage(commonMessages.privateLabel) }} </span>
								</template>
								<template v-else-if="collection.status === 'rejected'">
									<XIcon />
									<span> {{ formatMessage(commonMessages.rejectedLabel) }} </span>
								</template>
							</div>
						</div>
					</nuxt-link>
				</div>
				<div
					v-if="route.params.projectType === 'collections' && collections?.length === 0"
					class="error"
				>
					<UpToDate class="icon" />
					<br />
					<span v-if="auth.user && auth.user.id === user.id" class="preserve-lines text">
						<IntlFormatted :message-id="messages.profileNoCollectionsAuthLabel">
							<template #create-link="{ children }">
								<a
									class="link"
									@click.prevent="(event) => $refs.modal_collection_creation.show(event)"
								>
									<component :is="() => children" />
								</a>
							</template>
						</IntlFormatted>
					</span>
					<span v-else class="text">{{ formatMessage(messages.profileNoCollectionsLabel) }}</span>
				</div>
			</div>
			<div class="normal-page__sidebar">
				<div
					v-if="organizations?.length > 0"
					class="mb-4 rounded-2xl border border-solid border-surface-4 bg-surface-3 p-4 pt-3"
				>
					<h2 class="m-0 mb-2 text-lg text-contrast">
						{{ formatMessage(messages.profileOrganizations) }}
					</h2>
					<div class="flex flex-wrap gap-2">
						<nuxt-link
							v-for="org in sortedOrgs"
							:key="org.id"
							v-tooltip="org.name"
							class="organization"
							:to="`/organization/${org.slug}`"
						>
							<Avatar :src="org.icon_url" :alt="'Icon for ' + org.name" size="3rem" />
						</nuxt-link>
					</div>
				</div>
				<UserBadges
					:downloads="sumDownloads"
					:join-date="joinDate"
					:role="user.role"
					:badges="user.badges"
					:has-midas="hasActiveMidas(user)"
					:has-pride="hasPride26Badge(user)"
					:earliest-project-by-type="earliestProjectByType"
					class="mb-4 rounded-2xl border border-solid border-surface-4 bg-surface-3 p-4 pt-3"
				/>
				<AdPlaceholder v-if="!auth.user" />
			</div>
		</div>
	</div>
</template>
<script setup>
import {
	AffiliateIcon,
	BadgeCheckIcon,
	BoxIcon,
	CalendarIcon,
	ChartIcon,
	CheckIcon,
	ClipboardCopyIcon,
	CurrencyIcon,
	DownloadIcon,
	EditIcon,
	GlobeIcon,
	InfoIcon,
	LibraryIcon,
	LinkIcon,
	LockIcon,
	MoreVerticalIcon,
	ReportIcon,
	SaveIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	Combobox,
	commonMessages,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	IntlFormatted,
	NavTabs,
	NewModal,
	PageHeader,
	ProjectCard,
	ProjectCardList,
	useCompactNumber,
	useFormatDateTime,
	useFormatNumber,
	UserBadges,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import { isAdmin, isStaff, UserBadge } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { onServerPrefetch } from 'vue'

import UpToDate from '~/assets/images/illustrations/up_to_date.svg?component'
import AdPlaceholder from '~/components/ui/AdPlaceholder.vue'
import CollectionCreateModal from '~/components/ui/create/CollectionCreateModal.vue'
import ModalCreation from '~/components/ui/create/ProjectCreateModal.vue'
import { getSignInRouteObj } from '~/composables/auth.js'
import { reportUser } from '~/utils/report-helpers.ts'
import { hasActiveMidas, hasPride26Badge } from '~/utils/user-membership.ts'

const data = useNuxtApp()
const route = useNativeRoute()
const auth = await useAuth()
const cosmetics = useCosmetics()
const tags = useGeneratedState()
const config = useRuntimeConfig()
const queryClient = useQueryClient()

const { formatMessage } = useVIntl()
const formatNumber = useFormatNumber()
const { formatCompactNumber, formatCompactNumberPlural } = useCompactNumber()
const formatRelativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})

const { addNotification } = injectNotificationManager()

const messages = defineMessages({
	profileProjectsLabel: {
		id: 'profile.label.projects',
		defaultMessage: '{count} {countPlural, plural, one {project} other {projects}}',
	},
	profileDownloadsLabel: {
		id: 'profile.label.downloads',
		defaultMessage: '{count} {countPlural, plural, one {download} other {downloads}}',
	},
	collectionProjectsCount: {
		id: 'profile.collection.projects-count',
		defaultMessage: '{count, plural, one {# project} other {# projects}}',
	},
	profileJoinedLabel: {
		id: 'profile.label.joined',
		defaultMessage: 'Joined',
	},
	savingLabel: {
		id: 'profile.label.saving',
		defaultMessage: 'Saving...',
	},
	emailLabel: {
		id: 'profile.details.label.email',
		defaultMessage: 'Email',
	},
	emailVerifiedLabel: {
		id: 'profile.details.label.email-verified',
		defaultMessage: 'Email verified',
	},
	emailVerifiedTooltip: {
		id: 'profile.details.tooltip.email-verified',
		defaultMessage: 'Email verified',
	},
	emailNotVerifiedTooltip: {
		id: 'profile.details.tooltip.email-not-verified',
		defaultMessage: 'Email not verified',
	},
	authProvidersLabel: {
		id: 'profile.details.label.auth-providers',
		defaultMessage: 'Auth providers',
	},
	paymentMethodsLabel: {
		id: 'profile.details.label.payment-methods',
		defaultMessage: 'Payment methods',
	},
	hasPasswordLabel: {
		id: 'profile.details.label.has-password',
		defaultMessage: 'Has password',
	},
	hasTotpLabel: {
		id: 'profile.details.label.has-totp',
		defaultMessage: 'Has TOTP',
	},
	bioFallbackUser: {
		id: 'profile.bio.fallback.user',
		defaultMessage: 'A Modrinth user.',
	},
	bioFallbackCreator: {
		id: 'profile.bio.fallback.creator',
		defaultMessage: 'A Modrinth creator.',
	},
	collectionLabel: {
		id: 'profile.label.collection',
		defaultMessage: 'Collection',
	},
	profileProjectsFollowersStats: {
		id: 'profile.stats.projects-followers',
		defaultMessage:
			'{count, plural, one {<stat>{count}</stat> project follower} other {<stat>{count}</stat> project followers}}',
	},
	profileUserId: {
		id: 'profile.user-id',
		defaultMessage: 'User ID: {id}',
	},
	profileOrganizations: {
		id: 'profile.label.organizations',
		defaultMessage: 'Organizations',
	},
	profileBadges: {
		id: 'profile.label.badges',
		defaultMessage: 'Badges',
	},
	profileManageProjectsButton: {
		id: 'profile.button.manage-projects',
		defaultMessage: 'Manage projects',
	},
	profileMetaDescription: {
		id: 'profile.meta.description',
		defaultMessage: "Download {username}'s projects on Modrinth",
	},
	profileMetaDescriptionWithBio: {
		id: 'profile.meta.description-with-bio',
		defaultMessage: "{bio} - Download {username}'s projects on Modrinth",
	},
	profileNoProjectsLabel: {
		id: 'profile.label.no-projects',
		defaultMessage: 'This user has no projects!',
	},
	profileNoProjectsAuthLabel: {
		id: 'profile.label.no-projects-auth',
		defaultMessage:
			"You don't have any projects.\nWould you like to <create-link>create one</create-link>?",
	},
	profileNoCollectionsLabel: {
		id: 'profile.label.no-collections',
		defaultMessage: 'This user has no collections!',
	},
	profileNoCollectionsAuthLabel: {
		id: 'profile.label.no-collections-auth',
		defaultMessage:
			"You don't have any collections.\nWould you like to <create-link>create one</create-link>?",
	},
	billingButton: {
		id: 'profile.button.billing',
		defaultMessage: 'Manage user billing',
	},
	infoButton: {
		id: 'profile.button.info',
		defaultMessage: 'View user details',
	},
	analyticsButton: {
		id: 'profile.button.analytics',
		defaultMessage: 'View user analytics',
	},
	setAffiliateButton: {
		id: 'profile.button.set-affiliate',
		defaultMessage: 'Set as affiliate',
	},
	removeAffiliateButton: {
		id: 'profile.button.remove-affiliate',
		defaultMessage: 'Remove as affiliate',
	},
	affiliateLabel: {
		id: 'profile.label.affiliate',
		defaultMessage: 'Affiliate',
	},
	editRoleButton: {
		id: 'profile.button.edit-role',
		defaultMessage: 'Edit role',
	},
	userNotFoundError: {
		id: 'profile.error.not-found',
		defaultMessage: 'User not found',
	},
	officialAccount: {
		id: 'profile.official-account',
		defaultMessage: 'Official Modrinth account',
	},
	officialAccountBio: {
		id: 'profile.official-account.bio',
		defaultMessage:
			'The official user account of Modrinth. Get support at <support-link></support-link> or via email at <email></email>',
	},
})

const client = injectModrinthClient()

const userId = useRouteId('user')

const {
	data: user,
	error: userError,
	suspense: userSuspense,
} = useQuery({
	queryKey: computed(() => ['user', userId]),
	queryFn: () => client.labrinth.users_v3.get(userId),
})

watch(
	userError,
	(error) => {
		if (error) {
			const status = error.statusCode ?? error.status ?? 404
			showError({
				fatal: true,
				statusCode: status,
				message: formatMessage(messages.userNotFoundError),
			})
		}
	},
	{ immediate: true },
)

const { data: projects, suspense: projectsSuspense } = useQuery({
	queryKey: computed(() => ['user', userId, 'projects']),
	queryFn: async () => {
		const projects = await client.labrinth.users_v2.getProjects(userId)
		for (const project of projects) {
			project.categories = project.categories.concat(project.loaders)
			project.project_type = data.$getProjectTypeForUrl(
				project.project_type,
				project.categories,
				tags.value,
			)
		}
		return projects
	},
})

const { data: organizations, suspense: orgsSuspense } = useQuery({
	queryKey: computed(() => ['user', userId, 'organizations']),
	queryFn: () => client.labrinth.users_v2.getOrganizations(userId),
})

const { data: collections, suspense: collectionsSuspense } = useQuery({
	queryKey: computed(() => ['user', userId, 'collections']),
	queryFn: () => client.labrinth.users_v2.getCollections(userId),
})

onServerPrefetch(async () => {
	await Promise.allSettled([
		userSuspense(),
		projectsSuspense(),
		orgsSuspense(),
		collectionsSuspense(),
	])
})

const sortedOrgs = computed(() =>
	organizations.value ? [...organizations.value].sort((a, b) => a.name.localeCompare(b.name)) : [],
)

const isModrinthUser = computed(() => user.value?.id === '2REoufqX')

const sortedCollections = computed(() => {
	const list = collections.value
	if (!list?.length) return []
	return [...list].sort((a, b) => {
		const updatedB = new Date(b.updated).getTime()
		const updatedA = new Date(a.updated).getTime()
		if (updatedB !== updatedA) return updatedB - updatedA
		return new Date(b.created).getTime() - new Date(a.created).getTime()
	})
})

const title = computed(() => (user.value ? `${user.value.username} - Modrinth` : 'Modrinth'))
const description = computed(() =>
	user.value?.bio
		? formatMessage(messages.profileMetaDescriptionWithBio, {
				bio: user.value.bio,
				username: user.value.username,
			})
		: user.value
			? formatMessage(messages.profileMetaDescription, { username: user.value.username })
			: '',
)

useSeoMeta({
	title: () => title.value,
	description: () => description.value,
	ogTitle: () => title.value,
	ogDescription: () => description.value,
	ogImage: () => user.value?.avatar_url ?? 'https://cdn.modrinth.com/placeholder.png',
})

const projectTypes = computed(() => {
	const obj = {}

	if (collections.value?.length > 0) {
		obj.collection = true
	}

	for (const project of projects.value ?? []) {
		obj[project.project_type] = true
	}

	delete obj.project

	return Object.keys(obj)
})
const sumDownloads = computed(() => {
	let sum = 0

	for (const project of projects.value ?? []) {
		sum += project.downloads
	}

	return sum
})

const joinDate = computed(() => new Date(user.value.created))

async function copyId() {
	await navigator.clipboard.writeText(user.value.id)
}

const earliestProjectByType = computed(() => {
	const obj = {}

	for (const project of projects.value ?? []) {
		obj[project.project_type] = new Date(project.published)
	}

	return obj
})

async function copyPermalink() {
	await navigator.clipboard.writeText(`${config.public.siteUrl}/user/${user.value.id}`)
}

const isAffiliate = computed(() => user.value?.badges & UserBadge.AFFILIATE)
const isAdminViewing = computed(() => isAdmin(auth.value.user))
const userDetailsModal = useTemplateRef('userDetailsModal')

async function toggleAffiliate(id) {
	await client.labrinth.users_v2.patch(id, { badges: user.value.badges ^ (1 << 7) })
	queryClient.invalidateQueries({ queryKey: ['user', userId] })
}

const profileHeaderSummary = computed(() =>
	user.value?.bio
		? user.value.bio
		: (projects.value?.length ?? 0) === 0
			? formatMessage(messages.bioFallbackUser)
			: formatMessage(messages.bioFallbackCreator),
)

const profileHeaderLeading = computed(() => ({
	type: 'avatar',
	src: user.value?.avatar_url,
	alt: user.value?.username,
	avatarSize: isModrinthUser.value ? '64px' : '96px',
	circle: true,
}))

const profileHeaderBadges = computed(() => [
	...(isModrinthUser.value
		? [
				{
					id: 'official',
					label: formatMessage(messages.officialAccount),
					icon: BadgeCheckIcon,
					iconProps: {
						fill: 'var(--color-brand-highlight)',
					},
					tooltip: formatMessage(messages.officialAccount),
					class: 'border-brand-highlight bg-brand-highlight text-brand',
				},
			]
		: []),
	...(isAdminViewing.value && isAffiliate.value
		? [
				{
					id: 'affiliate',
					label: 'Affiliate',
					icon: AffiliateIcon,
					class: 'border-brand-highlight bg-brand-highlight text-brand',
				},
			]
		: []),
])

const profileHeaderMetadata = computed(() => {
	if (isModrinthUser.value) return []

	return [
		{
			id: 'projects',
			label: formatMessage(messages.profileProjectsLabel, {
				count: formatCompactNumber(projects.value?.length || 0),
				countPlural: formatCompactNumberPlural(projects.value?.length || 0),
			}),
			icon: BoxIcon,
		},
		{
			id: 'downloads',
			label: formatMessage(messages.profileDownloadsLabel, {
				count: formatCompactNumber(sumDownloads.value),
				countPlural: formatCompactNumberPlural(sumDownloads.value),
			}),
			icon: DownloadIcon,
			tooltip: formatNumber(sumDownloads.value),
		},
		{
			id: 'joined',
			label: `${formatMessage(messages.profileJoinedLabel)} ${formatRelativeTime(user.value.created)}`,
			icon: CalendarIcon,
			tooltip: formatDateTime(user.value.created),
		},
	]
})

const profileHeaderActions = computed(() => {
	if (!user.value) return []

	const viewer = auth.value.user
	const isSelf = viewer?.id === user.value.id

	return [
		...(isSelf
			? [
					{
						id: 'edit-profile',
						label: formatMessage(commonMessages.editButton),
						icon: EditIcon,
						to: '/settings/profile',
					},
				]
			: []),
		{
			id: 'more',
			label: 'More options',
			icon: MoreVerticalIcon,
			labelHidden: true,
			type: 'transparent',
			tooltip: 'More options',
			menuActions: [
				{
					id: 'manage-projects',
					label: formatMessage(messages.profileManageProjectsButton),
					icon: BoxIcon,
					action: () => navigateTo('/dashboard/projects'),
					shown: isSelf,
				},
				{
					divider: true,
					shown: isSelf,
				},
				{
					id: 'report',
					label: formatMessage(commonMessages.reportButton),
					icon: ReportIcon,
					action: () => (viewer ? reportUser(user.value.id) : navigateTo(getSignInRouteObj(route))),
					color: 'red',
					shown: viewer?.id !== user.value.id,
				},
				{
					id: 'copy-id',
					label: formatMessage(commonMessages.copyIdButton),
					icon: ClipboardCopyIcon,
					action: () => copyId(),
				},
				{
					id: 'copy-permalink',
					label: formatMessage(commonMessages.copyPermalinkButton),
					icon: ClipboardCopyIcon,
					action: () => copyPermalink(),
				},
				{
					divider: true,
					shown: viewer && isAdmin(viewer),
				},
				{
					id: 'open-billing',
					label: formatMessage(messages.billingButton),
					icon: CurrencyIcon,
					action: () => navigateTo(`/admin/billing/${user.value.id}`),
					shown: viewer && isStaff(viewer),
				},
				{
					id: 'toggle-affiliate',
					label: formatMessage(
						isAffiliate.value ? messages.removeAffiliateButton : messages.setAffiliateButton,
					),
					icon: AffiliateIcon,
					action: () => toggleAffiliate(user.value.id),
					shown: isAdminViewing.value,
					remainOnClick: true,
					color: isAffiliate.value ? 'red' : 'orange',
				},
				{
					id: 'open-info',
					label: formatMessage(messages.infoButton),
					icon: InfoIcon,
					action: () => userDetailsModal.value?.show(),
					shown: viewer && isStaff(viewer),
				},
				{
					id: 'open-analytics',
					label: formatMessage(messages.analyticsButton),
					icon: ChartIcon,
					action: () =>
						navigateTo({
							path: '/dashboard/analytics',
							query: { user: user.value.username || user.value.id },
						}),
					shown: viewer && isAdmin(viewer),
				},
				{
					id: 'edit-role',
					label: formatMessage(messages.editRoleButton),
					icon: EditIcon,
					action: () => openRoleEditModal(),
					shown: viewer && isAdmin(viewer),
				},
			],
		},
	]
})

const navLinks = computed(() => [
	{
		label: formatMessage(commonMessages.allProjectType),
		href: `/user/${user.value.username}`,
	},
	...projectTypes.value
		.map((x) => {
			return {
				label: formatMessage(getProjectTypeMessage(x, true)),
				href: `/user/${user.value.username}/${x}s`,
			}
		})
		.slice()
		.sort((a, b) => a.label.localeCompare(b.label)),
])

const selectedRole = ref(user.value?.role)
const isSavingRole = ref(false)

const roleOptions = [
	{ value: 'developer', label: 'Developer' },
	{ value: 'moderator', label: 'Moderator' },
	{ value: 'admin', label: 'Admin' },
]

const editRoleModal = useTemplateRef('editRoleModal')

const openRoleEditModal = () => {
	selectedRole.value = user.value.role
	editRoleModal.value?.show()
}

const cancelRoleEdit = () => {
	selectedRole.value = user.value.role
	editRoleModal.value?.hide()
}

function saveRoleEdit() {
	if (!selectedRole.value || selectedRole.value === user.value.role) {
		return
	}

	isSavingRole.value = true

	client.labrinth.users_v2
		.patch(user.value.id, { role: selectedRole.value })
		.then(() => {
			user.value.role = selectedRole.value

			editRoleModal.value?.hide()
		})
		.catch(() => {
			console.error('Failed to update user role:', error)

			addNotification({
				type: 'error',
				title: 'Failed to update role',
				message: 'An error occurred while updating the user role. Please try again.',
			})
		})
		.finally(() => {
			isSavingRole.value = false
		})
}
</script>
<script>
export default defineNuxtComponent({
	methods: {},
})
</script>

<style lang="scss" scoped>
.collections-grid {
	display: grid;
	grid-template-columns: repeat(2, 1fr);

	@media screen and (max-width: 800px) {
		grid-template-columns: repeat(1, 1fr);
	}

	gap: var(--gap-md);

	.collection-item {
		display: flex;
		flex-direction: column;
		gap: var(--gap-md);
		margin-bottom: 0px;
	}

	.description {
		flex-grow: 1;

		color: var(--color-text);
		font-size: 16px;
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

	.collection {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: var(--gap-md);

		.icon {
			width: 100% !important;
			height: 6rem !important;
			max-width: unset !important;
			max-height: unset !important;
			aspect-ratio: 1 / 1;
			object-fit: cover;
		}

		.details {
			display: flex;
			flex-direction: column;
			gap: var(--gap-sm);

			.title {
				color: var(--color-contrast);
				font-weight: 700;
				font-size: var(--font-size-lg);
				margin: 0;
			}
		}
	}
}
</style>
