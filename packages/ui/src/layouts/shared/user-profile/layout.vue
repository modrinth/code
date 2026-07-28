<template>
	<template v-if="user">
		<NewModal
			v-if="variant === 'web'"
			ref="editRoleModal"
			:header="formatMessage(messages.editRoleButton)"
		>
			<div class="flex w-80 flex-col gap-4">
				<Combobox
					v-model="selectedRole"
					:options="roleOptions"
					:placeholder="formatMessage(messages.selectRolePlaceholder)"
				/>
				<div class="flex justify-end gap-2">
					<ButtonStyled>
						<button type="button" @click="cancelRoleEdit">
							<XIcon />
							{{ formatMessage(commonMessages.cancelButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button
							type="button"
							:disabled="!selectedRole || selectedRole === user.role || isSavingRole"
							@click="saveRoleEdit"
						>
							<template v-if="isSavingRole">
								<SpinnerIcon class="animate-spin" />
								{{ formatMessage(messages.savingLabel) }}
							</template>
							<template v-else>
								<SaveIcon />
								{{ formatMessage(commonMessages.saveChangesButton) }}
							</template>
						</button>
					</ButtonStyled>
				</div>
			</div>
		</NewModal>

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
					<span>{{ user.auth_providers?.join(', ') || '—' }}</span>
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

		<NormalPage :sidebar="sidebarPosition">
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
					:projects-count="projects.length"
					:downloads="sumDownloads"
					@manage-projects="openPath('/dashboard/projects')"
					@report="reportProfile"
					@copy-id="copyId"
					@copy-permalink="copyPermalink"
					@open-billing="openPath(`/admin/billing/${user.id}`)"
					@toggle-affiliate="toggleAffiliate"
					@open-info="openUserDetails"
					@open-analytics="
						openPath(`/dashboard/analytics?user=${encodeURIComponent(user.username)}`)
					"
					@edit-role="openRoleEditModal"
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

			<div class="flex flex-col gap-4">
				<div v-if="navLinks.length > 2" class="max-w-full overflow-x-auto">
					<NavTabs :links="navLinks" replace />
				</div>

				<div class="flex flex-col gap-3">
					<ProjectCardList
						v-if="selectedProjectType !== 'collection' && filteredProjects.length > 0"
						:layout="displayMode"
					>
						<ProjectCard
							v-for="project in filteredProjects"
							:key="project.id"
							:link="projectLink(project)"
							:title="project.title"
							:icon-url="project.icon_url"
							:date-updated="project.updated"
							:downloads="project.downloads"
							:summary="project.description"
							:tags="[...project.categories, ...project.loaders]"
							:all-tags="[
								...project.categories,
								...project.loaders,
								...project.additional_categories,
							]"
							:followers="project.followers"
							:banner="project.gallery?.find((image) => image.featured)?.url"
							:color="project.color"
							:environment="{
								clientSide: project.client_side,
								serverSide: project.server_side,
							}"
							:layout="displayMode === 'list' ? 'list' : 'grid'"
							:status="project.status"
						/>
					</ProjectCardList>

					<EmptyState
						v-if="showProjectsEmptyState"
						type="empty"
						:heading="formatMessage(messages.profileNoProjectsLabel)"
						:description="
							isSelf ? formatMessage(messages.profileNoProjectsAuthDescription) : undefined
						"
					>
						<template v-if="isSelf" #actions>
							<ButtonStyled color="brand">
								<button type="button" @click="createProject">
									{{ formatMessage(messages.createProjectButton) }}
								</button>
							</ButtonStyled>
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
							<ButtonStyled color="brand">
								<button type="button" @click="createCollection">
									{{ formatMessage(messages.createCollectionButton) }}
								</button>
							</ButtonStyled>
						</template>
					</EmptyState>
				</div>
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
				<ButtonStyled color="brand">
					<button type="button" @click="retryQueries">
						{{ formatMessage(commonMessages.retryButton) }}
					</button>
				</ButtonStyled>
			</template>
		</EmptyState>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	BoxIcon,
	CheckIcon,
	GlobeIcon,
	LibraryIcon,
	LinkIcon,
	LockIcon,
	SaveIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import {
	isModrinthUser as checkIsModrinthUser,
	isOfficialAccount as checkIsOfficialAccount,
	UserBadge,
} from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import AutoLink from '#ui/components/base/AutoLink.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import Combobox from '#ui/components/base/Combobox.vue'
import EmptyState from '#ui/components/base/EmptyState.vue'
import IntlFormatted from '#ui/components/base/IntlFormatted.vue'
import NavTabs from '#ui/components/base/NavTabs.vue'
import SmartClickable from '#ui/components/base/SmartClickable.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import NormalPage from '#ui/components/page/NormalPage.vue'
import ProjectCard from '#ui/components/project/card/ProjectCard.vue'
import ProjectCardList from '#ui/components/project/ProjectCardList.vue'
import UserBadges from '#ui/components/user/UserBadges.vue'
import UserPageHeader from '#ui/components/user/UserPageHeader.vue'
import { defineMessages, useVIntl } from '#ui/composables'
import { injectAuth, injectNotificationManager, injectPageContext, injectTags } from '#ui/providers'
import { commonMessages, getProjectTypeTitleMessage } from '#ui/utils'

import { injectUserProfile } from './providers'
import {
	hasActivePride26Midas,
	hasPride26Badge,
	projectUserSorting,
	resolveProjectType,
} from './utils'

type DisplayMode = 'list' | 'grid' | 'gallery'
type ModalRef = {
	show: () => void
	hide: () => void
}
type ResolvedProject = Labrinth.Projects.v2.Project & {
	resolvedProjectType: string
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
		onCreateProject: undefined,
		onCreateCollection: undefined,
	},
)

const userProfile = injectUserProfile()
const auth = injectAuth()
const tags = injectTags(null)
const pageContext = injectPageContext()
const notificationManager = injectNotificationManager()
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
	savingLabel: {
		id: 'profile.label.saving',
		defaultMessage: 'Saving...',
	},
	editRoleButton: {
		id: 'profile.button.edit-role',
		defaultMessage: 'Edit role',
	},
	selectRolePlaceholder: {
		id: 'profile.role.select-placeholder',
		defaultMessage: 'Select a role',
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
	roleUpdateErrorTitle: {
		id: 'profile.role.update-error-title',
		defaultMessage: 'Failed to update role',
	},
	roleUpdateErrorDescription: {
		id: 'profile.role.update-error-description',
		defaultMessage: 'An error occurred while updating the user role. Please try again.',
	},
})

const userQuery = useQuery({
	queryKey: computed(() => ['user', props.userId]),
	queryFn: () => userProfile.getUser(props.userId),
	enabled: computed(() => Boolean(props.userId)),
	staleTime: 30_000,
})
const projectsQuery = useQuery({
	queryKey: computed(() => ['user', props.userId, 'projects']),
	queryFn: () => userProfile.getProjects(props.userId),
	enabled: computed(() => Boolean(props.userId)),
	staleTime: 30_000,
})
const organizationsQuery = useQuery({
	queryKey: computed(() => ['user', props.userId, 'organizations']),
	queryFn: () => userProfile.getOrganizations(props.userId),
	enabled: computed(() => Boolean(props.userId)),
	staleTime: 30_000,
})
const collectionsQuery = useQuery({
	queryKey: computed(() => ['user', props.userId, 'collections']),
	queryFn: () => userProfile.getCollections(props.userId),
	enabled: computed(() => Boolean(props.userId)),
	staleTime: 30_000,
})

const user = computed(() => userQuery.data.value)
const projects = computed<ResolvedProject[]>(() =>
	(projectsQuery.data.value ?? []).map((project) => ({
		...project,
		resolvedProjectType: resolveProjectType(project, tags?.loaders.value ?? []),
	})),
)
const organizations = computed(() => organizationsQuery.data.value ?? [])
const collections = computed(() => collectionsQuery.data.value ?? [])

const selectedProjectType = computed(() => {
	const projectType = props.projectType
	if (!projectType) return null
	if (projectType === 'collections' || projectType === 'collection') return 'collection'
	return projectType.endsWith('s') ? projectType.slice(0, -1) : projectType
})

const filteredProjects = computed(() => {
	const selected = selectedProjectType.value
	return projects.value
		.filter((project) => !selected || project.resolvedProjectType === selected)
		.slice()
		.sort(projectUserSorting)
})

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
	const types = new Set(projects.value.map((project) => project.resolvedProjectType))
	if (collections.value.length > 0) types.add('collection')
	types.delete('project')
	return [...types]
})

const navLinks = computed(() => {
	if (!user.value) return []
	const profilePath = `/user/${encodeURIComponent(props.userId)}`
	return [
		{
			label: formatMessage(commonMessages.allProjectType),
			href: profilePath,
		},
		...projectTypes.value
			.map((projectType) => ({
				label:
					projectType === 'collection'
						? formatMessage(messages.collectionsLabel)
						: formatMessage(getProjectTypeTitleMessage(projectType), { count: 2 }),
				href: `${profilePath}/${projectType}s`,
			}))
			.sort((first, second) => first.label.localeCompare(second.label)),
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
		const projectType = project.resolvedProjectType as EarlyAdopterProjectType
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
const editProfileLink = computed(() => linkTarget('/settings/profile'))

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

function projectLink(project: ResolvedProject): string | (() => void) {
	if (props.projectLinkMode === 'app') {
		return `/project/${project.id}`
	}
	return `/${project.resolvedProjectType}/${project.slug || project.id}`
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
const editRoleModal = ref<ModalRef | null>(null)
const selectedRole = ref<Labrinth.Users.v3.Role | null>(null)
const isSavingRole = ref(false)
const roleOptions = [
	{ value: 'developer', label: 'Developer' },
	{ value: 'moderator', label: 'Moderator' },
	{ value: 'admin', label: 'Admin' },
] satisfies { value: Labrinth.Users.v3.Role; label: string }[]

watch(
	user,
	(currentUser) => {
		selectedRole.value = currentUser?.role ?? null
	},
	{ immediate: true },
)

function openUserDetails(): void {
	userDetailsModal.value?.show()
}

function openRoleEditModal(): void {
	selectedRole.value = user.value?.role ?? null
	editRoleModal.value?.show()
}

function cancelRoleEdit(): void {
	selectedRole.value = user.value?.role ?? null
	editRoleModal.value?.hide()
}

async function toggleAffiliate(): Promise<void> {
	if (!user.value) return
	await userProfile.patchUser(user.value.id, {
		badges: user.value.badges ^ UserBadge.AFFILIATE,
	})
	await queryClient.invalidateQueries({ queryKey: ['user', props.userId] })
}

async function saveRoleEdit(): Promise<void> {
	if (!user.value || !selectedRole.value || selectedRole.value === user.value.role) return

	isSavingRole.value = true
	try {
		await userProfile.patchUser(user.value.id, { role: selectedRole.value })
		await queryClient.invalidateQueries({ queryKey: ['user', props.userId] })
		editRoleModal.value?.hide()
	} catch {
		notificationManager.addNotification({
			type: 'error',
			title: formatMessage(messages.roleUpdateErrorTitle),
			text: formatMessage(messages.roleUpdateErrorDescription),
		})
	} finally {
		isSavingRole.value = false
	}
}
</script>
