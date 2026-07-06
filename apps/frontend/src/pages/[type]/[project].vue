<template>
	<template v-if="project && projectV3Loaded">
		<Teleport v-if="flags.projectBackground" to="#fixed-background-teleport">
			<ProjectBackgroundGradient :project="project" />
		</Teleport>
		<template v-if="isSettings">
			<div v-if="canAccessSettings" class="normal-page no-sidebar">
				<div class="normal-page__header">
					<div
						class="mb-4 flex flex-wrap items-center gap-x-2 gap-y-3 border-0 border-b-[1px] border-solid border-divider pb-4 text-lg font-semibold"
					>
						<nuxt-link
							:to="`/${project.project_type}/${project.slug ? project.slug : project.id}`"
							class="flex items-center gap-2 hover:underline hover:brightness-[--hover-brightness]"
						>
							<Avatar :src="project.icon_url" size="32px" />
							{{ project.title }}
						</nuxt-link>
						<ChevronRightIcon />
						<span class="flex grow font-extrabold text-contrast">{{
							formatMessage(messages.settingsTitle)
						}}</span>
						<div class="flex gap-2">
							<ButtonStyled>
								<nuxt-link to="/dashboard/projects"
									><ListIcon /> {{ formatMessage(messages.visitProjectsDashboard) }}
								</nuxt-link>
							</ButtonStyled>
						</div>
					</div>
					<ProjectMemberHeader
						v-if="currentMember && false"
						:project="project"
						:versions="versions"
						:current-member="currentMember"
						:is-settings="isSettings"
						:set-processing="setProcessing"
						:all-members="allMembers"
						:update-members="invalidateProject"
						:auth="auth"
						:tags="tags"
					/>
				</div>
				<div class="normal-page__content">
					<NuxtPage />
				</div>
			</div>
		</template>

		<div v-else>
			<NewModal
				ref="modalLicense"
				:header="project.license.name ? project.license.name : formatMessage(messages.licenseTitle)"
			>
				<template #title>
					<Avatar :src="project.icon_url" :alt="project.title" class="icon" size="32px" no-shadow />
					<span class="text-lg font-extrabold text-contrast">
						{{ project.license.name ? project.license.name : formatMessage(messages.licenseTitle) }}
					</span>
				</template>
				<div
					class="markdown-body"
					v-html="
						renderString(licenseText).isEmpty
							? formatMessage(messages.loadingLicenseText)
							: renderString(licenseText)
					"
				/>
			</NewModal>
			<OpenInAppModal ref="openInAppModal" />
			<div
				class="over-the-top-download-animation"
				:class="{ 'animation-hidden': !overTheTopDownloadAnimation }"
			>
				<div>
					<div
						class="animation-ring-3 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight opacity-40"
					></div>

					<div
						class="animation-ring-2 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight opacity-60"
					></div>

					<div
						class="animation-ring-1 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight"
					>
						<DownloadIcon class="h-20 w-20 text-contrast" />
					</div>
				</div>
			</div>
			<ProjectDownloadModal
				ref="downloadModal"
				:project-id="routeProjectId"
				:download-reason="downloadReason"
				@download="triggerDownloadAnimation"
			/>
			<CollectionCreateModal ref="modal_collection" :project-ids="[project.id]" />
			<div
				class="new-page sidebar"
				:class="{
					'alt-layout': cosmetics.leftContentLayout,
					'checklist-open':
						showModerationChecklist &&
						!collapsedModerationChecklist &&
						!flags.alwaysShowChecklistAsPopup,
					'checklist-collapsed':
						showModerationChecklist &&
						collapsedModerationChecklist &&
						!flags.alwaysShowChecklistAsPopup,
				}"
			>
				<div class="normal-page__header relative my-4">
					<div class="mb-6">
						<ModerationProjectNags
							v-if="
								projectV3 &&
								currentMember &&
								(project.status === 'draft' || tags.rejectedStatuses.includes(project.status))
							"
							:project="project"
							:project-v3="projectV3"
							:versions="versions ?? undefined"
							:current-member="currentMember"
							:collapsed="collapsedChecklist"
							:route-name="route.name"
							:tags="tags"
							@toggle-collapsed="() => (collapsedChecklist = !collapsedChecklist)"
							@set-processing="setProcessing"
						/>
					</div>
					<ProjectPageHeader
						v-if="projectV3Loaded"
						:project="project"
						:project-v3="projectV3"
						:auth-user="auth.user"
						:sign-in-route="signInRouteObj"
						:collections="collections"
						:base-id="baseId"
						:collect-project="onUserCollectProject"
						:create-collection="(event) => modalCollection?.show(event)"
						:is-server-project="isServerProject"
						:show-status-badge="!!currentMember || project.status !== 'approved'"
						:show-edit-project="!!(auth.user && currentMember)"
						:primary-muted="!!currentMember || route.name === 'type-project-version-version'"
						:primary-label-hidden="!!(auth.user && currentMember)"
						:can-create-server="canCreateServerFrom"
						:show-quick-server-button="flags.showProjectPageQuickServerButton"
						:show-create-server-prompt="flags.showProjectPageCreateServersTooltip"
						:following="following"
						:saved="collections.some((x) => x.projects.includes(project.id))"
						:is-member="isMember"
						:is-staff="!!(auth.user && tags.staffRoles.includes(auth.user.role))"
						:show-moderation-checklist="showModerationChecklist"
						@category="(category) => router.push(`${projectSearchUrl}?f=categories:${category}`)"
						@primary="handleProjectHeaderPrimary"
						@create-server="dismissProjectHeaderCreateServerPrompt"
						@dismiss-create-server="dismissProjectHeaderCreateServerPrompt"
						@follow="followProjectFromHeader"
						@moderation-checklist="openModerationChecklistFromMenu"
						@report="reportProjectFromHeader"
						@copy-id="copyId"
						@copy-permalink="copyPermalink"
					/>
					<ProjectMemberHeader
						v-if="currentMember"
						:project="project"
						:versions="versions"
						:current-member="currentMember"
						:is-settings="isSettings"
						:route-name="route.name"
						:set-processing="setProcessing"
						:collapsed="collapsedChecklist"
						:toggle-collapsed="() => (collapsedChecklist = !collapsedChecklist)"
						:all-members="allMembers"
						:update-members="invalidateProject"
						:auth="auth"
						:tags="tags"
					/>
					<Admonition
						v-if="
							currentMember &&
							projectV3?.side_types_migration_review_status === 'pending' &&
							projectV3?.environment?.length === 1 &&
							projectV3?.environment[0] !== 'unknown'
						"
						type="warning"
						:header="
							formatMessage(
								hasEditDetailsPermission
									? messages.environmentMigrationTitle
									: messages.environmentMigrationNoPermissionTitle,
							)
						"
						class="mt-3"
					>
						{{
							formatMessage(
								hasEditDetailsPermission
									? messages.environmentMigrationMessage
									: messages.environmentMigrationNoPermissionMessage,
							)
						}}
						<nuxt-link
							to="/news/article/new-environments"
							target="_blank"
							class="mt-1 block w-fit font-semibold text-orange hover:underline"
						>
							{{ formatMessage(messages.environmentMigrationLink) }}
						</nuxt-link>
						<ButtonStyled v-if="hasEditDetailsPermission" color="orange">
							<button class="mt-3 w-fit" @click="() => projectEnvironmentModal.show()">
								<SettingsIcon /> {{ formatMessage(messages.reviewEnvironmentSettings) }}
							</button>
						</ButtonStyled>
					</Admonition>
					<MessageBanner v-if="project.status === 'archived'" message-type="warning" class="my-4">
						{{ formatMessage(messages.archivedMessage, { title: project.title }) }}
					</MessageBanner>
				</div>

				<div class="normal-page__sidebar">
					<ProjectSidebarServerInfo
						v-if="isServerProject && serverDataLoaded"
						:project-v3="projectV3"
						:tags="tags"
						:required-content="serverRequiredContent"
						:recommended-version="serverRecommendedVersion"
						:supported-versions="serverSupportedVersions"
						:loaders="serverModpackLoaders"
						:status-online="projectV3?.minecraft_java_server?.ping?.data != null"
						class="card flex-card"
					/>
					<ProjectSidebarCompatibility
						v-if="
							projectV3Loaded && !isServerProject && route.name !== 'type-project-version-version'
						"
						:project="project"
						:tags="tags"
						:project-v3="projectV3"
						class="card flex-card"
					/>
					<AdPlaceholder v-if="!auth.user && tags.approvedStatuses.includes(project.status)" />
					<ProjectSidebarLinks
						:project="project"
						:project-v3="projectV3"
						:link-target="$external()"
						class="card flex-card"
					/>
					<ProjectSidebarTags :project="project" class="card flex-card" />
					<ProjectSidebarCreators
						:organization="organization"
						:members="members"
						:org-link="(slug) => `/organization/${slug}`"
						:user-link="(username) => `/user/${username}`"
						class="card flex-card"
					/>
					<!-- TODO: Finish license modal and enable -->
					<ProjectSidebarDetails
						v-if="false"
						:project="project"
						:has-versions="versions.length > 0"
						:link-target="$external()"
						:show-followers="isServerProject"
						class="card flex-card"
					/>
					<div class="card flex-card">
						<h2>{{ formatMessage(detailsMessages.title) }}</h2>

						<div class="details-list">
							<div v-if="projectV3Loaded && !isServerProject" class="details-list__item">
								<BookTextIcon aria-hidden="true" />
								<div>
									{{ formatMessage(messages.licensedLabel) }}
									<a
										v-if="project.license.url"
										class="text-link hover:underline"
										:href="project.license.url"
										:target="$external()"
										rel="noopener nofollow ugc"
									>
										{{ licenseIdDisplay }}
										<ExternalIcon aria-hidden="true" class="external-icon ml-1 mt-[-1px] inline" />
									</a>
									<span
										v-else-if="
											project.license.id === 'LicenseRef-All-Rights-Reserved' ||
											!project.license.id.includes('LicenseRef')
										"
										class="text-link hover:underline"
										@click="(event) => getLicenseData(event)"
									>
										{{ licenseIdDisplay }}
									</span>
									<span v-else>{{ licenseIdDisplay }}</span>
								</div>
							</div>

							<div v-if="isServerProject" class="details-list__item">
								<HeartIcon aria-hidden="true" />
								<div>
									{{
										capitalizeString(
											formatMessage(commonMessages.projectFollowers, {
												count: project.followers,
											}),
										)
									}}
								</div>
							</div>
							<div
								v-if="project.approved"
								v-tooltip="formatDateTime(project.approved)"
								class="details-list__item"
							>
								<CalendarIcon aria-hidden="true" />
								<div>
									{{
										capitalizeString(
											formatMessage(detailsMessages.published, {
												date: publishedDate,
											}),
										)
									}}
								</div>
							</div>

							<div v-else v-tooltip="formatDateTime(project.published)" class="details-list__item">
								<CalendarIcon aria-hidden="true" />
								<div>
									{{
										capitalizeString(formatMessage(detailsMessages.created, { date: createdDate }))
									}}
								</div>
							</div>

							<div
								v-if="project.status === 'processing' && project.queued"
								v-tooltip="formatDateTime(project.queued)"
								class="details-list__item"
							>
								<ScaleIcon aria-hidden="true" />
								<div>
									{{
										capitalizeString(
											formatMessage(detailsMessages.submitted, {
												date: submittedDate,
											}),
										)
									}}
								</div>
							</div>

							<div
								v-if="versions.length > 0 && project.updated"
								v-tooltip="formatDateTime(project.updated)"
								class="details-list__item"
							>
								<VersionIcon aria-hidden="true" />
								<div>
									{{
										capitalizeString(formatMessage(detailsMessages.updated, { date: updatedDate }))
									}}
								</div>
							</div>
						</div>
					</div>
				</div>

				<div class="normal-page__content">
					<div class="mb-3 overflow-x-auto"><NavTabs :links="navLinks" replace class="mb-1" /></div>
					<NuxtPage @on-download="triggerDownloadAnimation" @delete-version="deleteVersion" />
				</div>
			</div>
		</div>

		<ClientOnly>
			<div
				v-if="auth.user && tags.staffRoles.includes(auth.user.role) && showModerationChecklist"
				class="moderation-checklist"
			>
				<ModerationChecklist
					:collapsed="collapsedModerationChecklist"
					@exit="setModerationChecklistOpen(false)"
					@toggle-collapsed="collapsedModerationChecklist = !collapsedModerationChecklist"
				/>
			</div>
		</ClientOnly>

		<template v-if="hasEditDetailsPermission">
			<ProjectEnvironmentModal ref="projectEnvironmentModal" />
		</template>
	</template>
</template>

<script setup>
import {
	BookTextIcon,
	CalendarIcon,
	ChevronRightIcon,
	DownloadIcon,
	ExternalIcon,
	HeartIcon,
	ListIcon,
	ScaleIcon,
	SettingsIcon,
	VersionIcon,
} from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	ButtonStyled,
	commonMessages,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	NavTabs,
	NewModal,
	OpenInAppModal,
	PROJECT_DEP_MARKER_QUERY,
	ProjectBackgroundGradient,
	ProjectEnvironmentModal,
	ProjectSidebarCompatibility,
	ProjectSidebarCreators,
	ProjectSidebarDetails,
	ProjectSidebarLinks,
	ProjectSidebarServerInfo,
	ProjectSidebarTags,
	provideProjectPageContext,
	useDebugLogger,
	useFormatDateTime,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import { capitalizeString, formatProjectType, renderString } from '@modrinth/utils'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { useLocalStorage } from '@vueuse/core'
import { onScopeDispose, readonly, ref, useTemplateRef, watch, watchEffect } from 'vue'

import { navigateTo } from '#app'
import AdPlaceholder from '~/components/ui/AdPlaceholder.vue'
import CollectionCreateModal from '~/components/ui/create/CollectionCreateModal.vue'
import MessageBanner from '~/components/ui/MessageBanner.vue'
import ModerationChecklist from '~/components/ui/moderation/checklist/ModerationChecklist.vue'
import ModerationProjectNags from '~/components/ui/moderation/ModerationProjectNags.vue'
import ProjectDownloadModal from '~/components/ui/ProjectDownloadModal/index.vue'
import ProjectPageHeader from '~/components/ui/ProjectPageHeader.vue'
import ProjectMemberHeader from '~/components/ui/ProjectMemberHeader.vue'
import { getSignInRouteObj } from '~/composables/auth.ts'
import { saveFeatureFlags } from '~/composables/featureFlags.ts'
import { STALE_TIME, STALE_TIME_LONG } from '~/composables/queries/project'
import { versionQueryOptions } from '~/composables/queries/version'
import { userCollectProject, userFollowProject } from '~/composables/user.js'
import { injectCurrentProjectId } from '~/providers/current-project.ts'
import {
	loadChecklistOpenState,
	saveChecklistOpenState,
} from '~/services/moderation-checklist-storage.ts'
import { useModerationQueue } from '~/services/moderation-queue.ts'
import { getReportPath, reportProject } from '~/utils/report-helpers.ts'

definePageMeta({
	key: (route) => `${route.params.project}`,
})

const data = useNuxtApp()
const route = useRoute()
const router = useRouter()
const signInRouteObj = computed(() => getSignInRouteObj(route))
const config = useRuntimeConfig()
const moderationQueue = useModerationQueue()
const notifications = injectNotificationManager()
const { addNotification } = notifications

const auth = await useAuth()
const user = await useUser()

// Route param for initial lookup (middleware caches by both slug and ID)
const routeProjectId = ref(useRouteId('project'))

const { createProjectDownloadUrl } = useCdnDownloadContext()

const downloadReason = ref('standalone')

function absorbDepQuery() {
	if (route.query.dep === PROJECT_DEP_MARKER_QUERY.dep) {
		downloadReason.value = 'dependency'
		if (import.meta.client) {
			const newQuery = { ...route.query }
			delete newQuery.dep
			void router.replace({ path: route.path, query: newQuery, hash: route.hash })
		}
	}
}

watch(() => route.query.dep, absorbDepQuery, { immediate: true })

const tags = useGeneratedState()
const flags = useFeatureFlags()
const cosmetics = useCosmetics()

const { formatMessage } = useVIntl()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})

const debug = useDebugLogger('DownloadModal')

const downloadModal = ref()
const openInAppModal = ref()
const overTheTopDownloadAnimation = ref()

const projectV3Loaded = computed(() => !projectV3Pending.value || projectV3.value != null)
const isServerProject = computed(() => projectV3.value?.minecraft_server != null)

const projectEnvironmentModal = useTemplateRef('projectEnvironmentModal')

const baseId = useId()

const serverProject = computed(() => ({
	name: project.value.title,
	slug: project.value.slug || project.value.id,
	numPlayers: projectV3.value?.minecraft_java_server?.ping?.data?.players_online,
	icon: project.value.icon_url,
	statusOnline: !!projectV3.value?.minecraft_java_server?.ping?.data,
	region: projectV3.value?.minecraft_server?.region,
}))

function handlePlayServerProject() {
	openInAppModal.value?.show({
		serverProject: serverProject.value,
	})
}

const formatRelativeTime = useRelativeTime()

const detailsMessages = defineMessages({
	title: {
		id: 'project.about.details.title',
		defaultMessage: 'Details',
	},
	licensed: {
		id: 'project.about.details.licensed',
		defaultMessage: 'Licensed {license}',
	},
	created: {
		id: 'project.about.details.created',
		defaultMessage: 'Created {date}',
	},
	submitted: {
		id: 'project.about.details.submitted',
		defaultMessage: 'Submitted {date}',
	},
	published: {
		id: 'project.about.details.published',
		defaultMessage: 'Published {date}',
	},
	updated: {
		id: 'project.about.details.updated',
		defaultMessage: 'Updated {date}',
	},
})

const messages = defineMessages({
	archivedMessage: {
		id: 'project.status.archived.message',
		defaultMessage:
			'{title} has been archived. {title} will not receive any further updates unless the author decides to unarchive the project.',
	},
	changelogTab: {
		id: 'project.navigation.changelog',
		defaultMessage: 'Changelog',
	},
	createServer: {
		id: 'project.actions.create-server',
		defaultMessage: 'Create a server',
	},
	descriptionTab: {
		id: 'project.description.title',
		defaultMessage: 'Description',
	},
	errorLoadingProject: {
		id: 'project.error.loading',
		defaultMessage: 'Error loading project data{message}',
	},
	environmentMigrationMessage: {
		id: 'project.environment.migration.message',
		defaultMessage:
			"We've just overhauled the Environments system on Modrinth and new options are now available. Please verify that the metadata is correct.",
	},
	environmentMigrationTitle: {
		id: 'project.environment.migration.title',
		defaultMessage: 'Please review environment metadata',
	},
	environmentMigrationNoPermissionMessage: {
		id: 'project.environment.migration-no-permission.message',
		defaultMessage:
			"We've just overhauled the Environments system on Modrinth and new options are now available. You don't have permission to modify these settings, but please let another member of the project know that the environment metadata needs to be verified.",
	},
	environmentMigrationNoPermissionTitle: {
		id: 'project.environment.migration-no-permission.title',
		defaultMessage: 'Environment metadata needs to be reviewed',
	},
	environmentMigrationLink: {
		id: 'project.environment.migration.learn-more',
		defaultMessage: 'Learn more about this change',
	},
	galleryTab: {
		id: 'project.gallery.title',
		defaultMessage: 'Gallery',
	},
	licenseErrorMessage: {
		id: 'project.license.error',
		defaultMessage: 'License text could not be retrieved.',
	},
	licenseTitle: {
		id: 'project.license.title',
		defaultMessage: 'License',
	},
	licensedLabel: {
		id: 'project.details.licensed',
		defaultMessage: 'Licensed',
	},
	loadingLicenseText: {
		id: 'project.license.loading',
		defaultMessage: 'Loading license text...',
	},
	moderationTab: {
		id: 'project.moderation.title',
		defaultMessage: 'Moderation',
	},
	pageNotFound: {
		id: 'project.error.page-not-found',
		defaultMessage: 'The page could not be found',
	},
	projectIconUpdated: {
		id: 'project.notification.icon-updated.title',
		defaultMessage: 'Project icon updated',
	},
	projectIconUpdatedMessage: {
		id: 'project.notification.icon-updated.message',
		defaultMessage: "Your project's icon has been updated.",
	},
	projectNotFound: {
		id: 'project.error.project-not-found',
		defaultMessage: 'Project not found',
	},
	projectUpdated: {
		id: 'project.notification.updated.title',
		defaultMessage: 'Project updated',
	},
	projectUpdatedMessage: {
		id: 'project.notification.updated.message',
		defaultMessage: 'Your project has been updated.',
	},
	reviewEnvironmentSettings: {
		id: 'project.environment.migration.review-button',
		defaultMessage: 'Review environment settings',
	},
	settingsTitle: {
		id: 'project.settings.title',
		defaultMessage: 'Settings',
	},
	versionsTab: {
		id: 'project.versions.title',
		defaultMessage: 'Versions',
	},
	visitProjectsDashboard: {
		id: 'project.settings.visit-dashboard',
		defaultMessage: 'Visit projects dashboard',
	},
})

const modalLicense = ref(null)
const modalCollection = useTemplateRef('modal_collection')
const licenseText = ref('')

const createdDate = computed(() =>
	project.value.published ? formatRelativeTime(project.value.published) : 'unknown',
)
const submittedDate = computed(() =>
	project.value.queued ? formatRelativeTime(project.value.queued) : 'unknown',
)
const publishedDate = computed(() =>
	project.value.approved ? formatRelativeTime(project.value.approved) : 'unknown',
)
const updatedDate = computed(() =>
	project.value.updated ? formatRelativeTime(project.value.updated) : 'unknown',
)

const licenseIdDisplay = computed(() => {
	const id = project.value.license.id

	if (id === 'LicenseRef-All-Rights-Reserved') {
		return 'ARR'
	} else if (id.includes('LicenseRef')) {
		return id.replaceAll('LicenseRef-', '').replaceAll('-', ' ')
	} else {
		return id
	}
})

async function getLicenseData(event) {
	modalLicense.value.show(event)

	try {
		const text = await client.labrinth.tags_v2.getLicenseText(project.value.license.id)
		licenseText.value = text.body || formatMessage(messages.licenseErrorMessage)
	} catch {
		licenseText.value = formatMessage(messages.licenseErrorMessage)
	}
}

const collections = computed(() =>
	user.value && user.value.collections ? user.value.collections : [],
)

if (
	!routeProjectId.value ||
	!(
		tags.value.projectTypes.find((x) => x.id === route.params.type) ||
		route.params.type === 'project'
	)
) {
	throw createError({
		fatal: false,
		statusCode: 404,
		message: formatMessage(messages.pageNotFound),
	})
}

// Use DI client for TanStack Query
const client = injectModrinthClient()
const queryClient = useQueryClient()

// V2 Project - hits middleware cache (uses route param for lookup)
const { data: projectRaw, error: projectV2Error } = useQuery({
	queryKey: computed(() => ['project', 'v2', routeProjectId.value]),
	queryFn: () => client.labrinth.projects_v2.get(routeProjectId.value),
	staleTime: STALE_TIME,
})

// Handle project not found - use showError since watch runs outside Nuxt context
watch(
	projectV2Error,
	(error) => {
		if (error) {
			// error.statusCode from ModrinthApiError, error.status as fallback
			const status = error.statusCode ?? error.status ?? 500
			showError({
				fatal: true,
				statusCode: status,
				message:
					status === 404
						? formatMessage(messages.projectNotFound)
						: formatMessage(messages.errorLoadingProject, {
								message: error.message ? `: ${error.message}` : '',
							}),
			})
		}
	},
	{ immediate: true },
)

// Transform project via computed
const project = computed(() => {
	if (!projectRaw.value) return null
	return {
		...projectRaw.value,
		actualProjectType: projectRaw.value.project_type,
		project_type: data.$getProjectTypeForUrl(
			projectRaw.value.project_type,
			projectRaw.value.loaders,
			tags.value,
		),
	}
})

// Use actual project ID for dependent queries (ensures cache consistency)
const projectId = computed(() => projectRaw.value?.id)

const sharedProjectId = injectCurrentProjectId(null)
if (sharedProjectId) {
	watchEffect(() => {
		sharedProjectId.value = projectId.value ?? undefined
	})
	onScopeDispose(() => {
		sharedProjectId.value = undefined
	})
}

// V3 Project
const {
	data: projectV3,
	error: _projectV3Error,
	isPending: projectV3Pending,
} = useQuery({
	queryKey: computed(() => ['project', 'v3', routeProjectId.value]),
	queryFn: () => client.labrinth.projects_v3.get(routeProjectId.value),
	staleTime: STALE_TIME,
})

// Server sidebar: modpack version + project for required content
const serverModpackVersionId = computed(() => {
	const content = projectV3.value?.minecraft_java_server?.content
	return content?.kind === 'modpack' ? content.version_id : null
})

const { data: serverModpackVersion, isPending: serverModpackVersionPending } = useQuery({
	queryKey: computed(() => ['version', 'v3', serverModpackVersionId.value]),
	queryFn: () => client.labrinth.versions_v3.getVersion(serverModpackVersionId.value),
	staleTime: STALE_TIME,
	enabled: computed(() => !!serverModpackVersionId.value),
})

const serverDataLoaded = computed(() => {
	if (!projectV3.value) return false
	if (serverModpackVersionId.value && serverModpackVersionPending.value) return false
	return true
})

const serverRequiredContent = computed(() => {
	const content = projectV3.value?.minecraft_java_server?.content
	if (!content || content.kind !== 'modpack') return null
	const primaryFile =
		serverModpackVersion.value?.files?.find((f) => f.primary) ??
		serverModpackVersion.value?.files?.[0]
	return {
		name: content.project_name ?? '',
		versionNumber: serverModpackVersion.value?.version_number ?? '',
		icon: content.project_icon,
		onclickName:
			content.project_id && content.project_id !== projectId.value
				? () => {
						navigateTo({
							path: `/project/${content.project_id}`,
							query: { ...PROJECT_DEP_MARKER_QUERY },
						})
					}
				: undefined,
		onclickVersion:
			content.project_id && content.project_id !== projectId.value
				? () => {
						navigateTo({
							path: `/project/${content.project_id}/version/${serverModpackVersion.value?.id}`,
							query: { ...PROJECT_DEP_MARKER_QUERY },
						})
					}
				: undefined,
		onclickDownload: primaryFile?.url
			? () =>
					navigateTo(createProjectDownloadUrl(primaryFile.url, { reason: 'dependency' }), {
						external: true,
					})
			: undefined,
		showCustomModpackTooltip: content.project_id === projectId.value,
	}
})

const serverRecommendedVersion = computed(() => {
	const content = projectV3.value?.minecraft_java_server?.content
	if (!content) return null

	if (content.kind === 'modpack') {
		return serverModpackVersion.value?.game_versions?.[0] ?? null
	}

	if (content.kind === 'vanilla') {
		return content.recommended_game_version ?? null
	}

	return null
})

const serverSupportedVersions = computed(() => {
	const content = projectV3.value?.minecraft_java_server?.content
	if (!content) return []

	if (content.kind === 'vanilla') {
		return content.supported_game_versions?.filter((v) => !!v) ?? []
	}

	return []
})

const serverModpackLoaders = computed(() => {
	if (!serverModpackVersion.value) return []
	return serverModpackVersion.value.mrpack_loaders ?? []
})

watch(serverModpackVersionId, (versionId) => {
	if (!versionId) return
	queryClient.prefetchQuery(versionQueryOptions.v3(versionId, client))
})

// Members
const { data: allMembersRaw, error: _membersError } = useQuery({
	queryKey: computed(() => ['project', projectId.value, 'members']),
	queryFn: () => client.labrinth.projects_v3.getMembers(projectId.value),
	staleTime: STALE_TIME,
	enabled: computed(() => !!projectId.value),
})

// Transform members via computed
const allMembers = computed(() => {
	if (!allMembersRaw.value) return []
	return allMembersRaw.value.map((it) => ({
		...it,
		avatar_url: it.user.avatar_url,
		name: it.user.username,
	}))
})

// Dependencies - lazy loaded client-side only
const dependenciesEnabled = ref(false)
const {
	data: dependenciesRaw,
	error: _dependenciesError,
	isFetching: dependenciesLoading,
} = useQuery({
	queryKey: computed(() => ['project', projectId.value, 'dependencies']),
	queryFn: () => client.labrinth.projects_v2.getDependencies(projectId.value),
	staleTime: STALE_TIME_LONG,
	enabled: computed(() => !!projectId.value && dependenciesEnabled.value),
})

const dependencies = computed(() => dependenciesRaw.value ?? null)

// V3 Versions - lazy loaded client-side only
const versionsEnabled = ref(false)
const {
	data: versionsV3,
	error: _versionsV3Error,
	isFetching: versionsV3Loading,
} = useQuery({
	queryKey: computed(() => ['project', projectId.value, 'versions', 'v3']),
	queryFn: () =>
		client.labrinth.versions_v3.getProjectVersions(projectId.value, {
			include_changelog: false,
			apiVersion: 3,
		}),
	staleTime: STALE_TIME_LONG,
	enabled: computed(() => !!projectId.value && versionsEnabled.value),
})

// Organization
// Only fetch organization if project belongs to one
const { data: organizationRaw } = useQuery({
	queryKey: computed(() => ['project', projectId.value, 'organization']),
	queryFn: () => client.labrinth.projects_v3.getOrganization(projectId.value),
	staleTime: STALE_TIME,
	enabled: computed(() => !!projectId.value && !!projectRaw.value?.organization),
})

// When project is removed from org, enabled becomes false but TanStack keeps stale data.
// Return null when the project no longer belongs to an organization.
const organization = computed(() => (projectRaw.value?.organization ? organizationRaw.value : null))

const isSettings = computed(() => route.name.startsWith('type-project-settings'))

// Transform versionsV3 to be same shape as versionsV2 for compatibility in project pages
const versionsRaw = computed(() => {
	return (versionsV3.value ?? []).map((version) => {
		const files = Array.isArray(version.files) ? version.files : []
		const gameVersions = Array.isArray(version.game_versions) ? version.game_versions : []
		const loaders = Array.isArray(version.loaders) ? version.loaders : []
		const isModpack = version.project_types?.includes('modpack')
		const mrpackLoaders = Array.isArray(version.mrpack_loaders) ? version.mrpack_loaders : []

		return {
			...version,
			files,
			game_versions: gameVersions,
			loaders: isModpack && mrpackLoaders.length ? mrpackLoaders : loaders,
		}
	})
})

// Apply version computations (slug generation, author lookup, etc.)
const versions = computed(() => {
	if (!versionsRaw.value.length || !allMembers.value.length) return versionsRaw.value
	return data.$computeVersions(versionsRaw.value, allMembers.value)
})

// Versions loading state
const versionsLoading = computed(() => versionsV3Loading.value)
const versionsLoaded = computed(() => versionsV3.value !== undefined || !!_versionsV3Error.value)

// Load versions on demand (client-side only)
function loadVersions() {
	debug('loadVersions called', {
		projectId: projectId.value,
		alreadyEnabled: versionsEnabled.value,
	})
	versionsEnabled.value = true
}

// Load dependencies on demand (client-side only)
function loadDependencies() {
	dependenciesEnabled.value = true
}

// Check if project has versions using the ID array from the V2 project
// This allows showing/hiding UI elements without loading full version data
const hasVersions = computed(() => (project.value?.versions?.length ?? 0) > 0)

async function invalidateProject() {
	await queryClient.invalidateQueries({ queryKey: ['project', 'v2', routeProjectId.value] })
	await queryClient.invalidateQueries({ queryKey: ['project', 'v3', routeProjectId.value] })
	if (routeProjectId.value !== projectId.value) {
		await queryClient.invalidateQueries({ queryKey: ['project', 'v2', projectId.value] })
		await queryClient.invalidateQueries({ queryKey: ['project', 'v3', projectId.value] })
	}
	// Prefix match — invalidates members, versions, dependencies, organization
	await queryClient.invalidateQueries({ queryKey: ['project', projectId.value] })
}

// Mutation for patching project data
const patchProjectMutation = useMutation({
	mutationFn: async ({ projectId, data }) => {
		await client.labrinth.projects_v2.edit(projectId, data)
		if (data.slug !== undefined && data.slug !== route.params.project) {
			routeProjectId.value = data.slug
			await navigateTo(
				{
					name: route.name,
					params: {
						type: route.params.type,
						project: data.slug,
					},
					query: route.query,
					hash: route.hash,
				},
				{ replace: true },
			)
		}
		return data
	},

	onMutate: async ({ projectId, data }) => {
		// Cancel outgoing refetches for both slug-based and ID-based cache keys
		// The query may be keyed by slug (routeProjectId.value) but we also have the actual UUID (projectId)
		await queryClient.cancelQueries({ queryKey: ['project', 'v2', routeProjectId.value] })
		if (routeProjectId.value !== projectId) {
			await queryClient.cancelQueries({ queryKey: ['project', 'v2', projectId] })
		}

		// Snapshot previous value from the active query (uses route param as key)
		const previousProject = queryClient.getQueryData(['project', 'v2', routeProjectId.value])

		// Optimistic update on the active query key
		queryClient.setQueryData(['project', 'v2', routeProjectId.value], (old) => {
			if (!old) return old
			return { ...old, ...data }
		})

		return { previousProject }
	},

	onError: (err, _variables, context) => {
		// Rollback on error using the active query key
		if (context?.previousProject) {
			queryClient.setQueryData(['project', 'v2', routeProjectId.value], context.previousProject)
		}
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err.message,
			type: 'error',
		})
	},

	onSettled: async () => {
		await invalidateProject()
	},
})

// Mutation for changing project status (setProcessing)
const patchStatusMutation = useMutation({
	mutationFn: async ({ projectId, status }) => {
		await client.labrinth.projects_v2.edit(projectId, { status })
	},

	onMutate: async ({ projectId, status }) => {
		// Cancel outgoing refetches for both slug-based and ID-based cache keys
		await queryClient.cancelQueries({ queryKey: ['project', 'v2', routeProjectId.value] })
		if (routeProjectId.value !== projectId) {
			await queryClient.cancelQueries({ queryKey: ['project', 'v2', projectId] })
		}

		// Snapshot previous value from the active query (uses route param as key)
		const previousProject = queryClient.getQueryData(['project', 'v2', routeProjectId.value])

		// Optimistic update on the active query key
		queryClient.setQueryData(['project', 'v2', routeProjectId.value], (old) => {
			if (!old) return old
			return { ...old, status }
		})

		return { previousProject }
	},

	onError: (err, _variables, context) => {
		// Rollback on error using the active query key
		if (context?.previousProject) {
			queryClient.setQueryData(['project', 'v2', routeProjectId.value], context.previousProject)
		}
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err.message,
			type: 'error',
		})
	},

	onSettled: async () => {
		await invalidateProject()
	},
})

// Mutation for patching V3 project data
const patchProjectV3Mutation = useMutation({
	mutationFn: async ({ projectId, data }) => {
		await client.labrinth.projects_v3.edit(projectId, data)
		return data
	},

	onMutate: async ({ projectId, data }) => {
		await queryClient.cancelQueries({ queryKey: ['project', 'v3', projectId] })

		const previousProject = queryClient.getQueryData(['project', 'v3', projectId])

		queryClient.setQueryData(['project', 'v3', projectId], (old) => {
			if (!old) return old
			const merged = { ...old }
			for (const [key, value] of Object.entries(data)) {
				if (
					value &&
					typeof value === 'object' &&
					!Array.isArray(value) &&
					merged[key] &&
					typeof merged[key] === 'object' &&
					!Array.isArray(merged[key])
				) {
					merged[key] = { ...merged[key], ...value }
				} else {
					merged[key] = value
				}
			}
			return merged
		})

		return { previousProject, projectId }
	},

	onError: (err, _variables, context) => {
		if (context?.previousProject) {
			queryClient.setQueryData(['project', 'v3', context.projectId], context.previousProject)
		}
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err.message,
			type: 'error',
		})
	},

	onSettled: async () => {
		await invalidateProject()
	},
})

// Mutation for patching project icon
const patchIconMutation = useMutation({
	mutationFn: async ({ projectId, icon }) => {
		const ext = icon.type.split('/')[icon.type.split('/').length - 1]
		await client.labrinth.projects_v3.changeIcon(projectId, icon, ext)
	},

	onSuccess: () => {
		addNotification({
			title: formatMessage(messages.projectIconUpdated),
			text: formatMessage(messages.projectIconUpdatedMessage),
			type: 'success',
		})
	},

	onError: (err) => {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err.message,
			type: 'error',
		})
	},

	onSettled: async () => {
		await invalidateProject()
	},
})

const createGalleryItemMutation = useMutation({
	mutationFn: async ({ projectId, file, title, description, featured, ordering }) => {
		const ext = file.type.split('/')[file.type.split('/').length - 1]
		await client.labrinth.projects_v2.createGalleryImage(projectId, file, {
			ext,
			featured: featured ?? false,
			title,
			description,
			ordering,
		})
	},

	onMutate: async ({ title, description, featured, ordering }) => {
		await queryClient.cancelQueries({ queryKey: ['project', 'v2', routeProjectId.value] })

		const previousProject = queryClient.getQueryData(['project', 'v2', routeProjectId.value])

		queryClient.setQueryData(['project', 'v2', routeProjectId.value], (old) => {
			if (!old) return old
			const newItem = {
				url: '',
				raw_url: '',
				featured: featured ?? false,
				title: title ?? '',
				description: description ?? '',
				created: new Date().toISOString(),
				ordering: ordering ?? old.gallery.length,
			}
			return {
				...old,
				gallery: [...old.gallery, newItem],
			}
		})

		return { previousProject }
	},

	onError: (err, _variables, context) => {
		if (context?.previousProject) {
			queryClient.setQueryData(['project', 'v2', routeProjectId.value], context.previousProject)
		}
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err.message,
			type: 'error',
		})
	},

	onSettled: async () => {
		await invalidateProject()
	},
})

const editGalleryItemMutation = useMutation({
	mutationFn: async ({ projectId, imageUrl, title, description, featured, ordering }) => {
		await client.labrinth.projects_v2.editGalleryImage(projectId, imageUrl, {
			featured: featured ?? false,
			title,
			description,
			ordering,
		})
	},

	onMutate: async ({ imageUrl, title, description, featured, ordering }) => {
		await queryClient.cancelQueries({ queryKey: ['project', 'v2', routeProjectId.value] })

		const previousProject = queryClient.getQueryData(['project', 'v2', routeProjectId.value])

		queryClient.setQueryData(['project', 'v2', routeProjectId.value], (old) => {
			if (!old) return old
			return {
				...old,
				gallery: old.gallery.map((item) => {
					if (item.url === imageUrl) {
						return {
							...item,
							title: title ?? item.title,
							description: description ?? item.description,
							featured: featured ?? item.featured,
							ordering: ordering ?? item.ordering,
						}
					}
					return item
				}),
			}
		})

		return { previousProject }
	},

	onError: (err, _variables, context) => {
		if (context?.previousProject) {
			queryClient.setQueryData(['project', 'v2', routeProjectId.value], context.previousProject)
		}
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err.message,
			type: 'error',
		})
	},

	onSettled: async () => {
		await invalidateProject()
	},
})

const deleteGalleryItemMutation = useMutation({
	mutationFn: async ({ projectId, imageUrl }) => {
		await client.labrinth.projects_v2.deleteGalleryImage(projectId, imageUrl)
	},

	onMutate: async ({ imageUrl }) => {
		await queryClient.cancelQueries({ queryKey: ['project', 'v2', routeProjectId.value] })

		const previousProject = queryClient.getQueryData(['project', 'v2', routeProjectId.value])

		queryClient.setQueryData(['project', 'v2', routeProjectId.value], (old) => {
			if (!old) return old
			return {
				...old,
				gallery: old.gallery.filter((item) => item.url !== imageUrl),
			}
		})

		return { previousProject }
	},

	onError: (err, _variables, context) => {
		if (context?.previousProject) {
			queryClient.setQueryData(['project', 'v2', routeProjectId.value], context.previousProject)
		}
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err.message,
			type: 'error',
		})
	},

	onSettled: async () => {
		await invalidateProject()
	},
})

// Members should be an array of all members, without the accepted ones, and with the user with the Owner role at the start
// The rest of the members should be sorted by role, then by name
const members = computed(() => {
	const acceptedMembers = allMembers.value.filter((x) => x.accepted)
	const owner = acceptedMembers.find((x) =>
		organization.value
			? organization.value.members?.some(
					(orgMember) => orgMember.user.id === x.user.id && orgMember.is_owner,
				)
			: x.is_owner,
	)

	const rest = acceptedMembers.filter((x) => !owner || x.user.id !== owner.user.id) || []

	rest.sort((a, b) => {
		if (a.role === b.role) {
			return a.user.username.localeCompare(b.user.username)
		} else {
			return a.role.localeCompare(b.role)
		}
	})

	return owner ? [owner, ...rest] : rest
})

const isMember = computed(
	() => auth.value.user && allMembers.value.some((x) => x.user.id === auth.value.user.id),
)

const currentMember = computed(() => {
	let val = auth.value.user ? allMembers.value.find((x) => x.user.id === auth.value.user.id) : null

	if (!val && auth.value.user && organization.value && organization.value.members) {
		val = organization.value.members.find((x) => x.user.id === auth.value.user.id)
	}

	if (
		!val &&
		auth.value.user &&
		project.value &&
		tags.value.staffRoles.includes(auth.value.user.role)
	) {
		val = {
			team_id: project.value.team_id,
			user: auth.value.user,
			role: auth.value.role,
			permissions: auth.value.user.role === 'admin' ? 1023 : 12,
			accepted: true,
			payouts_split: 0,
			avatar_url: auth.value.user.avatar_url,
			name: auth.value.user.username,
			staffOnly: true,
		}
	}

	return val
})

const canAccessSettings = computed(() => !!currentMember.value?.accepted)

const hasEditDetailsPermission = computed(() => {
	const EDIT_DETAILS = 1 << 2
	return (currentMember.value?.permissions & EDIT_DETAILS) === EDIT_DETAILS
})

watch(
	[isSettings, allMembers, canAccessSettings],
	() => {
		if (isSettings.value && allMembers.value.length > 0 && !canAccessSettings.value) {
			showError({
				fatal: true,
				statusCode: 401,
				statusMessage: 'Unauthorized',
			})
		}
	},
	{ flush: 'sync', immediate: true },
)

const projectTypeDisplay = computed(() => {
	if (!project.value) return ''
	const projectType = isServerProject.value ? 'minecraft_java_server' : project.value.project_type
	return formatProjectType(data.$getProjectTypeForDisplay(projectType, project.value.loaders))
})

const following = computed(() => {
	if (!user.value?.follows || !project.value) {
		return false
	}
	return !!user.value.follows.find((x) => x.id === project.value.id)
})

const title = computed(() =>
	project.value ? `${project.value.title} - Minecraft ${projectTypeDisplay.value}` : '',
)
const description = computed(() =>
	project.value
		? `${project.value.description} - Download the Minecraft ${projectTypeDisplay.value} ${
				project.value.title
			} by ${members.value.find((x) => x.is_owner)?.user?.username || 'a creator'} on Modrinth`
		: '',
)

const canCreateServerFrom = computed(() => {
	if (!project.value) return false
	return project.value.project_type === 'modpack' && project.value.server_side !== 'unsupported'
})

const projectSearchUrl = computed(
	() => `/discover/${isServerProject.value ? 'servers' : `${project.value?.project_type}s`}`,
)

const createCanonicalUrl = () =>
	project.value ? `https://modrinth.com/project/${project.value.id}` : undefined

useHead({
	link: [
		{
			rel: 'canonical',
			href: createCanonicalUrl,
		},
	],
})

if (!route.name.startsWith('type-project-settings')) {
	useSeoMeta({
		title: () => title.value,
		description: () => description.value,
		ogTitle: () => title.value,
		ogDescription: () => project.value?.description ?? '',
		ogImage: () => project.value?.icon_url ?? 'https://cdn.modrinth.com/placeholder.png',
		ogUrl: createCanonicalUrl,
		robots: () =>
			project.value?.status === 'approved' || project.value?.status === 'archived'
				? 'all'
				: 'noindex',
	})
} else {
	useSeoMeta({
		robots: 'noindex',
		ogUrl: createCanonicalUrl,
	})
}

const onUserCollectProject = useClientTry(userCollectProject)

function handleProjectHeaderPrimary(event) {
	if (isServerProject.value) {
		handlePlayServerProject()
	} else {
		downloadModal.value?.show(event)
	}
}

function dismissProjectHeaderCreateServerPrompt() {
	flags.value.showProjectPageCreateServersTooltip = false
	saveFeatureFlags()
}

function followProjectFromHeader() {
	if (!project.value) return
	userFollowProject(project.value)
}

function reportProjectFromHeader() {
	if (!project.value) return
	if (auth.value.user) {
		reportProject(project.value.id)
	} else {
		navigateTo(getSignInRouteObj(route, getReportPath('project', project.value.id)))
	}
}

watch(
	[versionsV3, _versionsV3Error],
	([data, error]) => {
		debug('versionsV3 query changed', {
			hasData: !!data,
			count: data?.length ?? 0,
			error: error?.message ?? null,
			projectId: projectId.value,
		})
	},
	{ immediate: true },
)

async function setProcessing() {
	// Guard against multiple submissions while mutation is pending
	if (patchStatusMutation.isPending.value) return

	startLoading()
	patchStatusMutation.mutate(
		{ projectId: project.value.id, status: 'processing' },
		{ onSettled: () => stopLoading() },
	)
}

async function patchProject(resData, quiet = false) {
	startLoading()

	return new Promise((resolve) => {
		patchProjectMutation.mutate(
			{ projectId: project.value.id, data: resData },
			{
				onSuccess: async () => {
					if (!quiet) {
						addNotification({
							title: formatMessage(messages.projectUpdated),
							text: formatMessage(messages.projectUpdatedMessage),
							type: 'success',
						})
					}
					resolve(true)
				},
				onError: () => resolve(false),
				onSettled: () => stopLoading(),
			},
		)
	})
}

async function patchProjectV3(resData, quiet = false) {
	startLoading()

	return new Promise((resolve) => {
		patchProjectV3Mutation.mutate(
			{ projectId: project.value.id, data: resData },
			{
				onSuccess: async () => {
					if (!quiet) {
						addNotification({
							title: formatMessage(messages.projectUpdated),
							text: formatMessage(messages.projectUpdatedMessage),
							type: 'success',
						})
					}
					resolve(true)
				},
				onError: () => resolve(false),
				onSettled: () => stopLoading(),
			},
		)
	})
}

async function patchIcon(icon) {
	startLoading()

	return new Promise((resolve) => {
		patchIconMutation.mutate(
			{ projectId: project.value.id, icon },
			{
				onSuccess: () => resolve(true),
				onError: () => resolve(false),
				onSettled: () => stopLoading(),
			},
		)
	})
}

async function createGalleryItem(file, title, description, featured, ordering) {
	startLoading()

	return new Promise((resolve) => {
		createGalleryItemMutation.mutate(
			{ projectId: project.value.id, file, title, description, featured, ordering },
			{
				onSuccess: () => resolve(true),
				onError: () => resolve(false),
				onSettled: () => stopLoading(),
			},
		)
	})
}

async function editGalleryItem(imageUrl, title, description, featured, ordering) {
	startLoading()

	return new Promise((resolve) => {
		editGalleryItemMutation.mutate(
			{ projectId: project.value.id, imageUrl, title, description, featured, ordering },
			{
				onSuccess: () => resolve(true),
				onError: () => resolve(false),
				onSettled: () => stopLoading(),
			},
		)
	})
}

async function deleteGalleryItem(imageUrl) {
	startLoading()

	return new Promise((resolve) => {
		deleteGalleryItemMutation.mutate(
			{ projectId: project.value.id, imageUrl },
			{
				onSuccess: () => resolve(true),
				onError: () => resolve(false),
				onSettled: () => stopLoading(),
			},
		)
	})
}

async function copyId() {
	await navigator.clipboard.writeText(project.value.id)
}

async function copyPermalink() {
	await navigator.clipboard.writeText(`${config.public.siteUrl}/project/${project.value.id}`)
}

const collapsedChecklist = ref(false)

const showModerationChecklist = ref(false)
const collapsedModerationChecklist = useLocalStorage('collapsed-moderation-checklist', false)

function consumeShowChecklistHistoryState() {
	if (!import.meta.client) return false
	if (!window.history?.state?.showChecklist) return false

	const state = { ...window.history.state }
	delete state.showChecklist
	window.history.replaceState(state, '', window.location.href)
	return true
}

function setModerationChecklistOpen(open, projectId = project.value?.id) {
	showModerationChecklist.value = open
	if (projectId) {
		void saveChecklistOpenState(projectId, open)
	}
}

function isProjectInActiveModerationQueue(projectId = project.value?.id) {
	return (
		!!projectId &&
		moderationQueue.isQueueMode &&
		moderationQueue.currentQueue.items.includes(projectId)
	)
}

async function openModerationChecklistFromMenu() {
	const projectId = project.value?.id
	if (!projectId) return

	await moderationQueue.ready
	if (!isProjectInActiveModerationQueue(projectId)) {
		await moderationQueue.setSingleProject(projectId)
	}

	setModerationChecklistOpen(true)
}

watch(
	() => project.value?.id,
	async (projectId, _previousProjectId, onCleanup) => {
		if (!import.meta.client || !projectId) return

		let cancelled = false
		onCleanup(() => {
			cancelled = true
		})

		const openedFromNavigation = consumeShowChecklistHistoryState()
		await moderationQueue.ready
		if (cancelled) return

		if (openedFromNavigation) {
			setModerationChecklistOpen(true)
			return
		}

		const storedOpen = await loadChecklistOpenState(projectId)
		if (cancelled) return

		if (storedOpen !== null) {
			showModerationChecklist.value = storedOpen
			return
		}

		const shouldRecoverFromQueue =
			moderationQueue.isQueueMode && moderationQueue.getCurrentProjectId() === projectId
		showModerationChecklist.value = shouldRecoverFromQueue

		if (shouldRecoverFromQueue) {
			void saveChecklistOpenState(projectId, true)
		}
	},
	{ immediate: true },
)

function triggerDownloadAnimation() {
	overTheTopDownloadAnimation.value = true
	setTimeout(() => (overTheTopDownloadAnimation.value = false), 500)
}

async function deleteVersion(id) {
	if (!id) return

	startLoading()

	await client.labrinth.versions_v3.deleteVersion(id)

	await invalidateProject()

	stopLoading()
}

const navLinks = computed(() => {
	const routeType = route.params.type || project.value.project_type
	const projectUrl = `/${routeType}/${project.value.slug ? project.value.slug : project.value.id}`

	const galleryCount =
		routeType === 'server'
			? project.value.gallery.filter((item) => item.name === '__mc_server_banner__').length
			: project.value.gallery.length

	return [
		{
			label: formatMessage(messages.descriptionTab),
			href: projectUrl,
		},
		{
			label: formatMessage(messages.galleryTab),
			href: `${projectUrl}/gallery`,
			shown: galleryCount > 0 || !!currentMember.value,
		},
		{
			label: formatMessage(messages.changelogTab),
			href: `${projectUrl}/changelog`,
			shown:
				hasVersions.value &&
				projectV3Loaded.value &&
				projectV3.value?.minecraft_server === undefined,
			onHover: loadVersions,
		},
		{
			label: formatMessage(messages.versionsTab),
			href: `${projectUrl}/versions`,
			shown:
				(hasVersions.value || !!currentMember.value) &&
				projectV3Loaded.value &&
				projectV3.value?.minecraft_server === undefined,
			subpages: [`${projectUrl}/version/`],
			onHover: loadVersions,
		},
		{
			label: formatMessage(messages.moderationTab),
			href: `${projectUrl}/moderation`,
			shown: !!currentMember.value,
		},
	]
})

provideProjectPageContext({
	// Data refs
	projectV2: project,
	projectV3,
	currentMember,
	allMembers,
	organization,
	// Lazy version loading
	versions,
	versionsLoading,
	versionsLoaded,
	// Lazy dependencies loading
	dependencies,
	dependenciesLoading: computed(() => dependenciesLoading.value),
	cdnDownloadReason: readonly(downloadReason),

	// Invalidate all project queries (auto-refetches active ones)
	invalidate: invalidateProject,

	// Lazy loading
	loadVersions,
	loadDependencies,

	// Mutation functions
	patchProject,
	patchProjectV3,
	patchIcon,
	setProcessing,

	// Gallery mutation functions
	createGalleryItem,
	editGalleryItem,
	deleteGalleryItem,
})
</script>

<style lang="scss" scoped>
.settings-header {
	display: flex;
	flex-direction: row;
	gap: var(--spacing-card-sm);
	align-items: center;
	margin-bottom: var(--spacing-card-bg);

	.settings-header__icon {
		flex-shrink: 0;
	}

	.settings-header__text {
		h1 {
			font-size: var(--font-size-md);
			margin-top: 0;
			margin-bottom: var(--spacing-card-sm);
		}
	}
}

.popout-checkbox {
	padding: var(--gap-sm) var(--gap-md);
	white-space: nowrap;

	&:hover {
		filter: brightness(0.95);
	}
}

.popout-heading {
	padding: var(--gap-sm) var(--gap-md);
	padding-bottom: 0;
	font-size: var(--font-size-nm);
	color: var(--color-secondary);
}

.menu-text {
	padding: 0 var(--gap-md);
	font-size: var(--font-size-nm);
	color: var(--color-secondary);
}

.menu-search {
	margin: var(--gap-sm) var(--gap-md);
	width: calc(100% - var(--gap-md) * 2);
}

.collections-list {
	max-height: 40rem;
	overflow-y: auto;
	background-color: var(--color-bg);
	border-radius: var(--radius-md);
	margin: var(--gap-sm) var(--gap-md);
	padding: var(--gap-sm);
}

.normal-page__info:empty {
	display: none;
}

.over-the-top-download-animation {
	position: fixed;
	z-index: 100;
	inset: 0;
	display: flex;
	justify-content: center;
	align-items: center;
	pointer-events: none;
	scale: 0.5;
	transition: all 0.5s ease-out;
	opacity: 1;

	&.animation-hidden {
		scale: 0.8;
		opacity: 0;

		.animation-ring-1 {
			width: 25rem;
			height: 25rem;
		}

		.animation-ring-2 {
			width: 50rem;
			height: 50rem;
		}

		.animation-ring-3 {
			width: 100rem;
			height: 100rem;
		}
	}

	> div {
		position: relative;
		display: flex;
		justify-content: center;
		align-items: center;
		width: fit-content;
		height: fit-content;

		> * {
			position: absolute;
			scale: 1;
			transition: all 0.2s ease-out;
			width: 20rem;
			height: 20rem;
		}
	}
}

.servers-popup {
	box-shadow:
		0 0 12px 1px rgba(0, 175, 92, 0.6),
		var(--shadow-floating);

	&::before {
		width: 0;
		height: 0;
		border-left: 6px solid transparent;
		border-right: 6px solid transparent;
		border-bottom: 6px solid var(--color-button-bg);
		content: ' ';
		position: absolute;
		top: -7px;
		left: 17px;
	}
	&::after {
		width: 0;
		height: 0;
		border-left: 5px solid transparent;
		border-right: 5px solid transparent;
		border-bottom: 5px solid var(--color-raised-bg);
		content: ' ';
		position: absolute;
		top: -5px;
		left: 18px;
	}
}

.moderation-checklist {
	position: fixed;
	bottom: 1rem;
	right: 1rem;
	overflow-y: auto;
	z-index: 50;
	transition: bottom 0.25s ease-in-out;

	> div {
		box-shadow: 0 0 15px rgba(0, 0, 0, 0.3);
	}
}

.new-page {
	column-gap: 1.5rem;
}
</style>

<style lang="scss">
body.floating-action-bar-shown .moderation-checklist {
	bottom: 6rem;
}
</style>
