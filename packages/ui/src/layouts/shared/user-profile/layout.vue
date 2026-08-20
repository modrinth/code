<template>
	<template v-if="user">
		<NewModal
			ref="blockUserModal"
			:header="formatMessage(messages.blockUserTitle, { username: user.username })"
			:closable="!isBlockingUser"
			fade="danger"
			max-width="500px"
		>
			<Admonition type="critical" :header="formatMessage(messages.blockUserAdmonitionTitle)">
				{{ formatMessage(messages.blockUserAdmonitionBody, { username: user.username }) }}
			</Admonition>

			<template #actions>
				<div class="flex justify-end gap-2">
					<Button
						type="outlined"
						native-type="button"
						:disabled="isBlockingUser"
						@click="blockUserModal?.hide()"
					>
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<Button
						type="colored"
						color="red"
						native-type="button"
						:disabled="isBlockingUser"
						@click="confirmBlockUser"
					>
						<SpinnerIcon v-if="isBlockingUser" class="animate-spin" />
						<BanIcon v-else />
						{{ formatMessage(messages.blockButton) }}
					</Button>
				</div>
			</template>
		</NewModal>

		<EditUserModal v-if="variant === 'web'" ref="editUserModal" :user="user" :user-id="userId" />

		<NewModal
			v-if="variant === 'web' && isStaffViewing"
			ref="userDetailsModal"
			:header="formatMessage(messages.userDetailsTitle)"
		>
			<div class="flex flex-col gap-3">
				<div v-if="isAdminViewing" class="flex flex-col gap-1">
					<span class="text-lg font-bold text-primary">
						{{ formatMessage(commonMessages.emailLabel) }}
					</span>
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

				<div v-else class="flex flex-col gap-1">
					<span class="text-lg font-bold text-primary">
						{{ formatMessage(messages.emailVerifiedLabel) }}
					</span>
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

				<div v-if="isAdminViewing" class="flex flex-col gap-1">
					<span class="text-lg font-bold text-primary">
						{{ formatMessage(messages.authProvidersLabel) }}
					</span>
					<ul class="flex flex-col gap-1 pl-4 leading-normal m-0">
						<li v-for="provider in user.auth_providers ?? []" :key="provider">
							<span>{{ authProviderNames[provider] ?? provider }}</span>
							<span v-if="provider === 'discord' && user.discord_id" class="ml-1">
								({{ user.discord_id }})
							</span>
							<template v-else-if="provider === 'github' && user.github_id">
								<span class="ml-1">(</span>
								<button
									type="button"
									class="m-0 appearance-none border-0 bg-transparent p-0 font-[inherit] text-link disabled:cursor-wait disabled:opacity-70"
									:disabled="isLoadingGithubProfile"
									@click="openGithubProfile"
								>
									{{
										isLoadingGithubProfile
											? formatMessage(messages.loadingGithubProfileLabel)
											: formatMessage(messages.viewGithubProfileLabel)
									}}
								</button>
								<span>)</span>
							</template>
							<span v-else-if="provider === 'steam' && user.steam_id" class="ml-1">
								({{ user.steam_id }})
							</span>
						</li>
					</ul>
				</div>

				<div v-if="isAdminViewing" class="flex flex-col gap-1">
					<span class="text-lg font-bold text-primary">
						{{ formatMessage(messages.paymentMethodsLabel) }}
					</span>
					<span>
						<template v-if="user.payout_data?.paypal_address">
							Paypal ({{ user.payout_data.paypal_address }}
							<template v-if="user.payout_data.paypal_country">
								- {{ user.payout_data.paypal_country }}
							</template>
							)
						</template>
						<template v-if="user.payout_data?.paypal_address && user.payout_data?.venmo_handle">
							,
						</template>
						<template v-if="user.payout_data?.venmo_handle">
							Venmo ({{ user.payout_data.venmo_handle }})
						</template>
						<template v-if="!user.payout_data?.paypal_address && !user.payout_data?.venmo_handle">
							—
						</template>
					</span>
				</div>

				<div class="flex flex-col gap-1">
					<span class="text-lg font-bold text-primary">
						{{ formatMessage(messages.hasPasswordLabel) }}
					</span>
					<span>
						{{
							user.has_password
								? formatMessage(commonMessages.yesLabel)
								: formatMessage(commonMessages.noLabel)
						}}
					</span>
				</div>

				<div class="flex flex-col gap-1">
					<span class="text-lg font-bold text-primary">
						{{ formatMessage(messages.hasTotpLabel) }}
					</span>
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

		<NormalPage :sidebar="sidebarPosition" :full-width="variant === 'app'">
			<template #header>
				<UserPageHeader
					:user="user"
					:summary="isModrinthUser ? null : profileHeaderSummary"
					:auth-user="auth.user.value"
					:edit-profile-link="editProfileLink"
					:is-modrinth-user="isModrinthUser"
					:is-official-account="isOfficialAccount"
					:show-affiliate-badge="isAdminViewing && isAffiliate"
					:is-affiliate="isAffiliate"
					:is-self="isSelf"
					:is-admin="isAdminViewing"
					:is-staff="isStaffViewing"
					:show-staff-actions="variant === 'web'"
					:is-blocked="isBlocked"
					:projects-count="projects.length"
					:downloads="sumDownloads"
					@manage-projects="openPath('/dashboard/projects')"
					@report="reportProfile"
					@block="handleBlockAction"
					@copy-id="copyId"
					@copy-permalink="copyPermalink"
					@open-billing="openPath(`/admin/billing/${user.id}`)"
					@toggle-affiliate="toggleAffiliate"
					@open-info="openUserDetails"
					@open-analytics="
						openPath(`/dashboard/analytics?user=${encodeURIComponent(user.username)}`)
					"
					@edit-user="editUserModal?.show()"
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
				</UserPageHeader>
			</template>

			<NavTabs v-if="navLinks.length > 2" :links="navLinks" replace page-nav />

			<div class="flex flex-col gap-3">
				<ProjectList
					v-if="selectedProjectType !== 'collection' && filteredProjects.length > 0"
					:projects="filteredProjects"
					:layout="displayMode"
					:link-mode="projectLinkMode"
					show-status
				>
					<template v-if="$slots['project-actions']" #actions="{ project }">
						<slot name="project-actions" :project="project" />
					</template>
				</ProjectList>

				<EmptyState
					v-if="showProjectsEmptyState"
					type="empty"
					:heading="formatMessage(messages.profileNoProjectsLabel)"
					:description="
						isSelf ? formatMessage(messages.profileNoProjectsAuthDescription) : undefined
					"
				>
					<template v-if="isSelf" #actions>
						<Button type="colored" color="brand" native-type="button" @click="createProject">
							{{ formatMessage(messages.createProjectButton) }}
						</Button>
					</template>
				</EmptyState>

				<ProjectCardList
					v-if="selectedProjectType === null || selectedProjectType === 'collection'"
					layout="grid"
				>
					<SmartClickable
						v-for="collection in sortedCollections"
						:key="collection.id"
						class="h-full w-full"
					>
						<template #clickable>
							<AutoLink
								:to="collectionLink(collection.id)"
								class="no-click-animation custom-focus-indicator rounded-xl no-outline"
							/>
						</template>
						<div
							class="smart-clickable:outline-on-focus smart-clickable:highlight-on-hover flex h-full w-full flex-col gap-4 overflow-hidden rounded-2xl border-[1px] border-solid border-surface-4 bg-surface-3 p-4 text-left transition-all"
						>
							<div class="grid grid-cols-[auto_1fr] gap-4">
								<Avatar :src="collection.icon_url" size="64px" no-shadow />
								<div class="flex min-w-0 flex-col gap-2">
									<h2
										class="smart-clickable:underline-on-hover m-0 truncate text-lg font-semibold text-contrast"
									>
										{{ collection.name }}
									</h2>
									<div class="flex items-center gap-1">
										<LibraryIcon aria-hidden="true" />
										{{ formatMessage(messages.collectionLabel) }}
									</div>
								</div>
							</div>
							<div class="grow text-primary">
								{{ collection.description }}
							</div>
							<div class="mt-auto flex flex-wrap items-center gap-4">
								<div class="flex items-center gap-1">
									<BoxIcon />
									{{
										formatMessage(messages.collectionProjectsCount, {
											count: collection.projects.length,
										})
									}}
								</div>
								<div class="flex items-center gap-1">
									<template v-if="collection.status === 'listed'">
										<GlobeIcon />
										{{ formatMessage(commonMessages.publicLabel) }}
									</template>
									<template v-else-if="collection.status === 'unlisted'">
										<LinkIcon />
										{{ formatMessage(commonMessages.unlistedLabel) }}
									</template>
									<template v-else-if="collection.status === 'private'">
										<LockIcon />
										{{ formatMessage(commonMessages.privateLabel) }}
									</template>
									<template v-else-if="collection.status === 'rejected'">
										<XIcon />
										{{ formatMessage(commonMessages.rejectedLabel) }}
									</template>
								</div>
							</div>
						</div>
					</SmartClickable>
				</ProjectCardList>

				<EmptyState
					v-if="showCollectionsEmptyState"
					type="empty"
					:heading="formatMessage(messages.profileNoCollectionsLabel)"
					:description="
						isSelf ? formatMessage(messages.profileNoCollectionsAuthDescription) : undefined
					"
				>
					<template v-if="isSelf" #actions>
						<Button type="colored" color="brand" native-type="button" @click="createCollection">
							{{ formatMessage(messages.createCollectionButton) }}
						</Button>
					</template>
				</EmptyState>
			</div>

			<template #sidebar>
				<div class="flex flex-col" :class="{ 'gap-4': variant === 'web' }">
					<div v-if="sortedOrganizations.length > 0" :class="sidebarSectionClass">
						<h2 class="m-0 mb-2 text-lg font-semibold text-contrast">
							{{ formatMessage(messages.profileOrganizations) }}
						</h2>
						<div class="flex flex-wrap gap-2">
							<AutoLink
								v-for="organization in sortedOrganizations"
								:key="organization.id"
								v-tooltip="organization.name"
								:to="organizationLink(organization.slug)"
								link-class="!inline-flex"
							>
								<Avatar
									:src="organization.icon_url"
									:alt="`Icon for ${organization.name}`"
									size="3rem"
								/>
							</AutoLink>
						</div>
					</div>

					<UserBadges
						:downloads="sumDownloads"
						:join-date="new Date(user.created)"
						:role="user.role"
						:badges="user.badges"
						:has-midas="hasMidas"
						:has-pride="hasPride26Badge(user)"
						:earliest-project-by-type="earliestProjectByType"
						:class="sidebarSectionClass"
					/>

					<slot name="sidebar" />
				</div>
			</template>
		</NormalPage>
	</template>

	<div v-else class="flex min-h-[24rem] items-center justify-center p-6">
		<EmptyState
			type="error"
			:heading="formatMessage(messages.userNotFoundError)"
			:description="formatMessage(messages.userLoadErrorDescription)"
		>
			<template #actions>
				<Button type="colored" color="brand" native-type="button" @click="retryQueries">
					{{ formatMessage(commonMessages.retryButton) }}
				</Button>
			</template>
		</EmptyState>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	BanIcon,
	BoxIcon,
	CheckIcon,
	GlobeIcon,
	LibraryIcon,
	LinkIcon,
	LockIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import {
	getPrimaryProjectType,
	isModrinthUser as checkIsModrinthUser,
	isOfficialAccount as checkIsOfficialAccount,
	UserBadge,
} from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import Admonition from '#ui/components/base/Admonition.vue'
import AutoLink from '#ui/components/base/AutoLink.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import { Button } from '#ui/components/base/buttons'
import EmptyState from '#ui/components/base/EmptyState.vue'
import IntlFormatted from '#ui/components/base/IntlFormatted.vue'
import NavTabs from '#ui/components/base/NavTabs.vue'
import SmartClickable from '#ui/components/base/SmartClickable.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import NormalPage from '#ui/components/page/NormalPage.vue'
import ProjectCardList from '#ui/components/project/ProjectCardList.vue'
import ProjectList from '#ui/components/project/ProjectList.vue'
import UserBadges from '#ui/components/user/UserBadges.vue'
import UserPageHeader from '#ui/components/user/UserPageHeader.vue'
import { defineMessages, useVIntl } from '#ui/composables'
import {
	injectAuth,
	injectModrinthClient,
	injectNotificationManager,
	injectPageContext,
} from '#ui/providers'
import {
	catalogProjectTypes,
	commonMessages,
	filterProjectsByType,
	getProjectTypeTitleMessage,
	parseProjectTypeRouteParam,
	sortProjectTypes,
} from '#ui/utils'

import EditUserModal from './components/edit-user-modal.vue'
import { blockedUsersQueryKey, injectUserProfile } from './providers'
import { hasActivePride26Midas, hasPride26Badge, projectUserSorting } from './utils'

type DisplayMode = 'list' | 'grid' | 'gallery'
type ModalRef = {
	show: () => void
	hide: () => void
}
type EarlyAdopterProjectType =
	| 'modpack'
	| 'resourcepack'
	| 'plugin'
	| 'datapack'
	| 'shader'
	| 'server'

const props = withDefaults(
	defineProps<{
		userId: string
		projectType?: string
		displayMode?: DisplayMode
		sidebarPosition?: 'left' | 'right'
		variant?: 'web' | 'app'
		siteUrl?: string
		externalNavigation?: boolean
		projectLinkMode?: 'website' | 'app'
		editProfileLink?: string | (() => void)
		onCreateProject?: (event?: MouseEvent) => void
		onCreateCollection?: (event?: MouseEvent) => void
	}>(),
	{
		projectType: undefined,
		displayMode: 'list',
		sidebarPosition: 'right',
		variant: 'web',
		siteUrl: 'https://modrinth.com',
		externalNavigation: false,
		projectLinkMode: 'website',
		editProfileLink: undefined,
		onCreateProject: undefined,
		onCreateCollection: undefined,
	},
)

const userProfile = injectUserProfile()
const auth = injectAuth()
const pageContext = injectPageContext()
const notificationManager = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const route = useRoute()
const router = useRouter()
const { formatMessage } = useVIntl()
const sidebarSectionClass = computed(() =>
	props.variant === 'app'
		? 'border-0 border-b-[1px] border-solid border-[--brand-gradient-border] p-4'
		: 'rounded-2xl border border-solid border-surface-4 bg-surface-3 p-4',
)

const messages = defineMessages({
	collectionProjectsCount: {
		id: 'profile.collection.projects-count',
		defaultMessage: '{count, plural, one {# project} other {# projects}}',
	},
	userDetailsTitle: {
		id: 'profile.details.title',
		defaultMessage: 'User details',
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
	viewGithubProfileLabel: {
		id: 'profile.details.label.view-github-profile',
		defaultMessage: 'View profile',
	},
	loadingGithubProfileLabel: {
		id: 'profile.details.label.loading-github-profile',
		defaultMessage: 'Loading...',
	},
	githubProfileErrorTitle: {
		id: 'profile.details.error.github-profile-title',
		defaultMessage: 'Unable to open GitHub profile',
	},
	githubProfileErrorMessage: {
		id: 'profile.details.error.github-profile-message',
		defaultMessage: 'The GitHub profile could not be retrieved. Please try again.',
	},
	githubPopupBlockedMessage: {
		id: 'profile.details.error.github-popup-blocked',
		defaultMessage: 'Allow pop-ups for Modrinth, then try again.',
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
	collectionsLabel: {
		id: 'project-type.collection.plural',
		defaultMessage: 'Collections',
	},
	profileOrganizations: {
		id: 'profile.label.organizations',
		defaultMessage: 'Organizations',
	},
	profileNoProjectsLabel: {
		id: 'profile.label.no-projects',
		defaultMessage: 'This user has no projects!',
	},
	profileNoProjectsAuthDescription: {
		id: 'profile.label.no-projects-auth-description',
		defaultMessage: "You don't have any projects yet.",
	},
	createProjectButton: {
		id: 'profile.button.create-project',
		defaultMessage: 'Create a project',
	},
	profileNoCollectionsLabel: {
		id: 'profile.label.no-collections',
		defaultMessage: 'This user has no collections!',
	},
	profileNoCollectionsAuthDescription: {
		id: 'profile.label.no-collections-auth-description',
		defaultMessage: "You don't have any collections yet.",
	},
	createCollectionButton: {
		id: 'profile.button.create-collection',
		defaultMessage: 'Create a collection',
	},
	userNotFoundError: {
		id: 'profile.error.not-found',
		defaultMessage: 'User not found',
	},
	userLoadErrorDescription: {
		id: 'profile.error.load-description',
		defaultMessage: 'The user profile could not be loaded.',
	},
	officialAccountBio: {
		id: 'profile.official-account.bio',
		defaultMessage:
			'The official user account of Modrinth. Get support at <support-link></support-link> or via email at <email></email>',
	},
	blockButton: {
		id: 'profile.button.block',
		defaultMessage: 'Block',
	},
	unblockUserSuccessTitle: {
		id: 'profile.unblock-user.success-title',
		defaultMessage: 'User unblocked',
	},
	unblockUserSuccessDescription: {
		id: 'profile.unblock-user.success-description',
		defaultMessage: '{username} has been unblocked.',
	},
	unblockUserErrorTitle: {
		id: 'profile.unblock-user.error-title',
		defaultMessage: 'Failed to unblock user',
	},
	unblockUserErrorDescription: {
		id: 'profile.unblock-user.error-description',
		defaultMessage: 'An error occurred while unblocking this user. Please try again.',
	},
	blockUserTitle: {
		id: 'profile.block-user.title',
		defaultMessage: 'Block {username}',
	},
	blockUserAdmonitionTitle: {
		id: 'profile.block-user.admonition-title',
		defaultMessage: 'Are you sure you want to block this user?',
	},
	blockUserAdmonitionBody: {
		id: 'profile.block-user.admonition-body',
		defaultMessage:
			'{username} will not be able to send you friend requests, invite you to shared instances or invite you to Modrinth Hosting servers.',
	},
	blockUserSuccessTitle: {
		id: 'profile.block-user.success-title',
		defaultMessage: 'User blocked',
	},
	blockUserSuccessDescription: {
		id: 'profile.block-user.success-description',
		defaultMessage: '{username} has been blocked.',
	},
	blockUserErrorTitle: {
		id: 'profile.block-user.error-title',
		defaultMessage: 'Failed to block user',
	},
	blockUserErrorDescription: {
		id: 'profile.block-user.error-description',
		defaultMessage: 'An error occurred while blocking this user. Please try again.',
	},
})

const canLoadProfile = computed(() => props.userId.length > 0)
const userQuery = useQuery({
	queryKey: computed(() => ['user', props.userId]),
	queryFn: () => userProfile.getUser(props.userId),
	enabled: canLoadProfile,
	placeholderData: (previousData) => previousData,
	staleTime: 30_000,
})
const projectsQuery = useQuery({
	queryKey: computed(() => ['user', props.userId, 'projects']),
	queryFn: () => userProfile.getProjects(props.userId),
	enabled: canLoadProfile,
	placeholderData: (previousData) => previousData,
	staleTime: 30_000,
})
const organizationsQuery = useQuery({
	queryKey: computed(() => ['user', props.userId, 'organizations']),
	queryFn: () => userProfile.getOrganizations(props.userId),
	enabled: canLoadProfile,
	placeholderData: (previousData) => previousData,
	staleTime: 30_000,
})
const collectionsQuery = useQuery({
	queryKey: computed(() => ['user', props.userId, 'collections']),
	queryFn: () => userProfile.getCollections(props.userId),
	enabled: canLoadProfile,
	placeholderData: (previousData) => previousData,
	staleTime: 30_000,
})
const blockedUsersQuery = useQuery({
	queryKey: computed(() => blockedUsersQueryKey(auth.user.value?.id)),
	queryFn: userProfile.getBlockedUsers,
	enabled: computed(() => Boolean(auth.user.value)),
	staleTime: 30_000,
})

const user = computed(() => userQuery.data.value)
const projects = computed(() => projectsQuery.data.value ?? [])
watch(
	() => projectsQuery.data.value,
	(projects) => {
		if (props.projectLinkMode !== 'app') return

		for (const project of projects ?? []) {
			for (const identifier of [project.id, project.slug]) {
				if (identifier) {
					queryClient.setQueryData(['projects', 'summary', identifier], project)
				}
			}
		}
	},
	{ immediate: true },
)
const organizations = computed(() => organizationsQuery.data.value ?? [])
const collections = computed(() => collectionsQuery.data.value ?? [])
const isBlocked = computed(() =>
	user.value ? (blockedUsersQuery.data.value ?? []).includes(user.value.id) : false,
)

const selectedProjectType = computed(() => parseProjectTypeRouteParam(props.projectType))

const filteredProjects = computed(() =>
	filterProjectsByType(projects.value, selectedProjectType.value).slice().sort(projectUserSorting),
)

const sortedOrganizations = computed(() =>
	organizations.value.slice().sort((first, second) => first.name.localeCompare(second.name)),
)
const sortedCollections = computed(() =>
	collections.value.slice().sort((first, second) => {
		const updatedDifference = new Date(second.updated).getTime() - new Date(first.updated).getTime()
		if (updatedDifference !== 0) return updatedDifference
		return new Date(second.created).getTime() - new Date(first.created).getTime()
	}),
)

const projectTypes = computed(() => {
	const types = catalogProjectTypes(projects.value)
	if (collections.value.length > 0) types.push('collection')
	return sortProjectTypes(types)
})

const navLinks = computed(() => {
	if (!user.value) return []
	const profilePath = `/user/${encodeURIComponent(props.userId)}`
	return [
		{
			label: formatMessage(commonMessages.allProjectType),
			href: profilePath,
		},
		...projectTypes.value.map((projectType) => ({
			label:
				projectType === 'collection'
					? formatMessage(messages.collectionsLabel)
					: formatMessage(getProjectTypeTitleMessage(projectType), { count: 2 }),
			href: `${profilePath}/${projectType}s`,
		})),
	]
})

const sumDownloads = computed(() =>
	projects.value.reduce((total, project) => total + project.downloads, 0),
)
const profileHeaderSummary = computed(() => {
	if (!user.value) return ''
	if (user.value.bio) return user.value.bio
	return projects.value.length === 0
		? formatMessage(messages.bioFallbackUser)
		: formatMessage(messages.bioFallbackCreator)
})
const earliestProjectByType = computed(() => {
	const earliest = {} as Record<EarlyAdopterProjectType, Date>
	for (const project of projects.value) {
		const projectType = getPrimaryProjectType(project) as EarlyAdopterProjectType
		const published = new Date(project.published)
		if (!earliest[projectType] || published < earliest[projectType]) {
			earliest[projectType] = published
		}
	}
	return earliest
})

const isModrinthUser = computed(() => checkIsModrinthUser(user.value?.id))
const isOfficialAccount = computed(() => checkIsOfficialAccount(user.value?.id))
const isSelf = computed(() => auth.user.value?.id === user.value?.id)
const isAdminViewing = computed(() => auth.user.value?.role === 'admin')
const isStaffViewing = computed(
	() => auth.user.value?.role === 'admin' || auth.user.value?.role === 'moderator',
)
const isAffiliate = computed(() => Boolean((user.value?.badges ?? 0) & UserBadge.AFFILIATE))
const hasMidas = computed(
	() => Boolean((user.value?.badges ?? 0) & UserBadge.MIDAS) || hasActivePride26Midas(user.value),
)
const showProjectsEmptyState = computed(
	() =>
		selectedProjectType.value !== 'collection' &&
		filteredProjects.value.length === 0 &&
		(selectedProjectType.value !== null || collections.value.length === 0),
)
const showCollectionsEmptyState = computed(
	() => selectedProjectType.value === 'collection' && collections.value.length === 0,
)

const normalizedSiteUrl = computed(() => props.siteUrl.replace(/\/$/, ''))
const editProfileLink = computed(() => props.editProfileLink ?? linkTarget('/settings/profile'))

const authProviderNames = {
	github: 'GitHub',
	discord: 'Discord',
	microsoft: 'Microsoft',
	gitlab: 'GitLab',
	google: 'Google',
	steam: 'Steam',
	paypal: 'PayPal',
}
const isLoadingGithubProfile = ref(false)

function externalUrl(path: string): string {
	return `${normalizedSiteUrl.value}${path.startsWith('/') ? path : `/${path}`}`
}

function linkTarget(path: string): string | (() => void) {
	if (!props.externalNavigation) return path
	return () => pageContext.openExternalUrl(externalUrl(path))
}

function openPath(path: string): void {
	const target = linkTarget(path)
	if (typeof target === 'function') {
		target()
	} else {
		void router.push(target)
	}
}

function organizationLink(slug: string): string | (() => void) {
	return linkTarget(`/organization/${encodeURIComponent(slug)}`)
}

function collectionLink(id: string): string | (() => void) {
	return linkTarget(`/collection/${encodeURIComponent(id)}`)
}

async function copyId(): Promise<void> {
	if (user.value) await navigator.clipboard.writeText(user.value.id)
}

async function copyPermalink(): Promise<void> {
	if (user.value) {
		await navigator.clipboard.writeText(externalUrl(`/user/${user.value.id}`))
	}
}

async function openGithubProfile() {
	const githubId = user.value?.github_id
	if (!githubId || isLoadingGithubProfile.value) return

	const profileWindow = window.open('about:blank', '_blank')
	if (!profileWindow) {
		notificationManager.addNotification({
			type: 'error',
			title: formatMessage(messages.githubProfileErrorTitle),
			text: formatMessage(messages.githubPopupBlockedMessage),
		})
		return
	}

	profileWindow.opener = null
	isLoadingGithubProfile.value = true

	try {
		const githubUser = await client.request<{ login?: string }>(`/${githubId}`, {
			api: 'https://api.github.com',
			version: 'user',
			method: 'GET',
			headers: { 'Content-Type': '' },
			skipAuth: true,
		})

		if (!githubUser?.login) {
			throw new Error('GitHub user response did not include a login')
		}

		profileWindow.location.replace(`https://github.com/${encodeURIComponent(githubUser.login)}`)
	} catch (error) {
		profileWindow.close()
		console.error('Failed to retrieve GitHub profile:', error)
		notificationManager.addNotification({
			type: 'error',
			title: formatMessage(messages.githubProfileErrorTitle),
			text: formatMessage(messages.githubProfileErrorMessage),
		})
	} finally {
		isLoadingGithubProfile.value = false
	}
}

function reportProfile(): void {
	if (!user.value) return
	const reportPath = `/report?item=user&itemID=${encodeURIComponent(user.value.id)}`
	if (props.externalNavigation) {
		pageContext.openExternalUrl(externalUrl(reportPath))
	} else if (auth.user.value) {
		void router.push(reportPath)
	} else {
		void auth.requestSignIn(route.fullPath)
	}
}

function createProject(event?: MouseEvent): void {
	if (props.onCreateProject) {
		props.onCreateProject(event)
	} else {
		openPath('/dashboard/projects')
	}
}

function createCollection(event?: MouseEvent): void {
	if (props.onCreateCollection) {
		props.onCreateCollection(event)
	} else {
		openPath('/dashboard/collections')
	}
}

async function retryQueries(): Promise<void> {
	await Promise.allSettled([
		userQuery.refetch(),
		projectsQuery.refetch(),
		organizationsQuery.refetch(),
		collectionsQuery.refetch(),
	])
}

const userDetailsModal = ref<ModalRef | null>(null)
const editUserModal = ref<InstanceType<typeof EditUserModal> | null>(null)
const blockUserModal = ref<ModalRef | null>(null)
const isBlockingUser = ref(false)
const isUnblockingUser = ref(false)

function openUserDetails(): void {
	userDetailsModal.value?.show()
}

async function handleBlockAction(): Promise<void> {
	if (!auth.user.value) {
		await auth.requestSignIn(route.fullPath)
		return
	}

	if (isBlocked.value) {
		await unblockCurrentUser()
		return
	}

	blockUserModal.value?.show()
}

async function confirmBlockUser(): Promise<void> {
	if (!user.value || isBlockingUser.value) return

	const blockedUser = user.value
	const authUserId = auth.user.value?.id
	isBlockingUser.value = true
	try {
		await userProfile.blockUser(blockedUser.id)
		queryClient.setQueryData<Labrinth.BlockedUsers.v3.BlockedUserId[]>(
			blockedUsersQueryKey(authUserId),
			(blockedUsers = []) =>
				blockedUsers.includes(blockedUser.id) ? blockedUsers : [...blockedUsers, blockedUser.id],
		)
		blockUserModal.value?.hide()
		notificationManager.addNotification({
			type: 'success',
			title: formatMessage(messages.blockUserSuccessTitle),
			text: formatMessage(messages.blockUserSuccessDescription, {
				username: blockedUser.username,
			}),
		})
	} catch {
		notificationManager.addNotification({
			type: 'error',
			title: formatMessage(messages.blockUserErrorTitle),
			text: formatMessage(messages.blockUserErrorDescription),
		})
	} finally {
		isBlockingUser.value = false
	}
}

async function unblockCurrentUser(): Promise<void> {
	if (!user.value || isUnblockingUser.value) return

	const blockedUser = user.value
	const authUserId = auth.user.value?.id
	isUnblockingUser.value = true
	try {
		await userProfile.unblockUser(blockedUser.id)
		queryClient.setQueryData<Labrinth.BlockedUsers.v3.BlockedUserId[]>(
			blockedUsersQueryKey(authUserId),
			(blockedUsers = []) => blockedUsers.filter((userId) => userId !== blockedUser.id),
		)
		notificationManager.addNotification({
			type: 'success',
			title: formatMessage(messages.unblockUserSuccessTitle),
			text: formatMessage(messages.unblockUserSuccessDescription, {
				username: blockedUser.username,
			}),
		})
	} catch {
		notificationManager.addNotification({
			type: 'error',
			title: formatMessage(messages.unblockUserErrorTitle),
			text: formatMessage(messages.unblockUserErrorDescription),
		})
	} finally {
		isUnblockingUser.value = false
	}
}

async function toggleAffiliate(): Promise<void> {
	if (!user.value) return
	await userProfile.patchUser(user.value.id, {
		badges: user.value.badges ^ UserBadge.AFFILIATE,
	})
	await queryClient.invalidateQueries({ queryKey: ['user', props.userId] })
}
</script>
