<template>
	<div class="overflow-hidden rounded-2xl">
		<div class="bg-bg-raised p-4">
			<div
				class="flex w-full flex-col items-start justify-between gap-3 sm:flex-row sm:items-center sm:gap-0"
			>
				<span class="text-md flex flex-col gap-2 sm:flex-row sm:items-center">
					<span class="flex items-center gap-2">
						<span class="text-secondary">Reported for</span>
						<span class="font-semibold text-contrast">
							{{ formattedReportType }}
						</span>
					</span>
					<span class="flex items-center gap-2">
						<span class="hidden text-secondary sm:inline">By</span>
						<span class="text-secondary sm:hidden">Reporter:</span>
						<nuxt-link
							:to="`/user/${report.reporter_user.username}`"
							target="_blank"
							class="inline-flex flex-row items-center gap-1 transition-colors duration-100 ease-in-out hover:text-brand"
						>
							<Avatar
								:src="report.reporter_user.avatar_url"
								circle
								size="1.75rem"
								class="flex-shrink-0"
							/>
							<span class="truncate">{{ report.reporter_user.username }}</span>
						</nuxt-link>
					</span>
				</span>

				<div class="flex flex-row items-center gap-2 self-end sm:self-auto">
					<span
						v-tooltip="formatDateTime(report.created)"
						class="cursor-help whitespace-nowrap text-sm text-secondary"
					>
						{{ formatRelativeTime(report.created) }}
					</span>
					<div class="flex items-center gap-2">
						<ButtonStyled circular>
							<button v-tooltip="'Copy ID'" @click="copyId">
								<ClipboardCopyIcon />
							</button>
						</ButtonStyled>
						<ButtonStyled circular>
							<a
								v-tooltip="'Open in new tab'"
								:href="`/moderation/reports/${props.report.id}`"
								target="_blank"
							>
								<ExternalIcon />
							</a>
						</ButtonStyled>
					</div>
				</div>
			</div>

			<div class="my-4 h-px bg-surface-5" />

			<div class="flex items-center justify-between">
				<div class="flex items-center gap-4">
					<Avatar
						:src="reportItemAvatarUrl"
						:circle="report.item_type === 'user'"
						size="4rem"
						:class="[
							'flex-shrink-0 border border-surface-5 bg-surface-4 !shadow-none',
							report.item_type !== 'user' && 'rounded-2xl',
						]"
					/>

					<div v-if="report.item_type === 'user'" class="flex flex-col gap-1.5">
						<NuxtLink
							:to="`/user/${report.user?.username}`"
							target="_blank"
							class="text-base font-semibold text-contrast hover:underline"
						>
							{{ report.user?.username || 'Unknown User' }}
						</NuxtLink>

						<span
							v-if="report.user?.created"
							v-tooltip="formatDateTime(report.user.created)"
							class="cursor-help text-sm text-secondary"
						>
							Joined {{ formatRelativeTime(report.user.created) }}
						</span>
					</div>

					<div v-else class="flex flex-col gap-1.5">
						<div class="flex flex-wrap items-center gap-2">
							<NuxtLink
								v-if="report.item_type !== 'shared-instance'"
								:to="reportItemUrl"
								target="_blank"
								class="text-base font-semibold text-contrast hover:underline"
							>
								{{ reportItemTitle }}
							</NuxtLink>
							<span v-else class="text-base font-semibold text-contrast">
								{{ reportItemTitle }}
							</span>

							<div
								v-if="report.project?.project_type"
								class="flex items-center gap-1 rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1"
							>
								<component
									:is="getProjectTypeIcon(report.project.project_type as any)"
									aria-hidden="true"
									class="h-4 w-4"
								/>
								<span class="text-sm font-medium text-secondary">
									{{ formatProjectType(report.project.project_type, true) }}
								</span>
							</div>

							<span
								v-if="report.item_type === 'version' && report.version"
								class="text-sm text-secondary"
							>
								{{ report.version.files.find((f) => f.primary)?.filename || 'Unknown Version' }}
							</span>
							<span
								v-if="
									report.item_type === 'shared-instance' &&
									report.shared_instance_version_id !== undefined
								"
								class="text-sm text-secondary"
							>
								Version {{ report.shared_instance_version_id }}
							</span>
							<CopyCode v-if="report.item_type === 'shared-instance'" :text="report.item_id" />
							<span
								v-if="report.item_type === 'shared-instance' && sharedInstanceQuarantined"
								class="bg-orange-highlight inline-flex items-center gap-1 rounded-full border border-solid border-orange px-2.5 py-1 text-sm font-semibold text-orange"
							>
								<LockIcon class="size-4" />
								Quarantined
							</span>
						</div>

						<div v-if="report.target" class="flex items-center gap-1">
							<Avatar
								:src="report.target.avatar_url"
								size="1.5rem"
								circle
								class="border border-surface-5 bg-surface-4 !shadow-none"
							/>
							<NuxtLink
								:to="`/${report.target.type}/${report.target.slug}`"
								target="_blank"
								class="text-sm font-medium text-secondary hover:underline"
							>
								{{ report.target.name }}
							</NuxtLink>
						</div>
						<div
							v-else-if="report.item_type === 'shared-instance' && sharedInstanceDetails"
							class="flex items-center gap-1"
						>
							<Avatar
								:src="sharedInstanceDetails.owner.avatar_url"
								size="1.5rem"
								circle
								class="border border-surface-5 bg-surface-4 !shadow-none"
							/>
							<NuxtLink
								:to="`/user/${sharedInstanceDetails.owner.username}`"
								target="_blank"
								class="text-sm font-medium text-secondary hover:underline"
							>
								{{ sharedInstanceDetails.owner.username }}
							</NuxtLink>
						</div>
					</div>
				</div>
			</div>
		</div>
		<CollapsibleRegion
			v-model:collapsed="isThreadCollapsed"
			:expand-text="expandText"
			collapse-text="Collapse thread"
		>
			<div class="bg-surface-2 pt-2">
				<ThreadView
					v-if="threadWithReportBody"
					ref="reportThread"
					:thread="threadWithReportBody"
					:quick-replies="reportQuickReplies"
					:quick-reply-context="report"
					:closed="reportClosed"
					@update-thread="updateThread"
				>
					<template #afterMessages>
						<template v-if="report.item_type === 'shared-instance'">
							<div
								v-if="sharedInstanceLoading"
								class="flex items-center gap-2 border-0 border-t border-solid border-surface-4 px-4 py-6 text-secondary"
							>
								<LoaderCircleIcon class="size-5 animate-spin" />
								Loading shared instance details…
							</div>
							<div
								v-else-if="sharedInstanceError"
								class="flex flex-col items-start gap-3 border-0 border-t border-solid border-surface-4 px-4 py-6"
							>
								<div class="flex flex-col gap-1">
									<span class="font-semibold text-contrast">
										Shared instance details could not be loaded
									</span>
									<span class="text-sm text-secondary">{{ sharedInstanceError }}</span>
								</div>
								<ButtonStyled type="outlined">
									<button @click="loadSharedInstanceDetails">Try again</button>
								</ButtonStyled>
							</div>
							<SharedInstanceReportContext
								v-else-if="sharedInstanceDetails"
								:details="sharedInstanceDetails"
								:ban-pending="sharedInstanceBanPending"
								:load-version-content="loadSharedInstanceVersionContent"
								@ban-owner="banSharedInstanceOwner"
								@content-error="showSharedInstanceContentError"
							/>
						</template>
					</template>
					<template #closedActions>
						<ButtonStyled v-if="isStaff(auth.user)" color="green">
							<button class="mt-2 w-full gap-2 sm:w-auto" @click="reopenReport()">
								<CheckCircleIcon class="size-4" />
								Reopen Thread
							</button>
						</ButtonStyled>
					</template>
					<template #additionalActions="{ hasReply }">
						<template v-if="isStaff(auth.user)">
							<ButtonStyled v-if="hasReply" color="red">
								<button class="w-full gap-2 sm:w-auto" @click="closeReport(true)">
									<CheckCircleIcon class="size-4" />
									Reply and close
								</button>
							</ButtonStyled>
							<ButtonStyled v-else color="red">
								<button class="w-full gap-2 sm:w-auto" @click="closeReport()">
									<CheckCircleIcon class="size-4" />
									Close report
								</button>
							</ButtonStyled>
						</template>
					</template>
				</ThreadView>
			</div>
		</CollapsibleRegion>
	</div>
</template>
<script setup lang="ts">
import type { Labrinth, SharedInstances } from '@modrinth/api-client'
import {
	CheckCircleIcon,
	ClipboardCopyIcon,
	ExternalIcon,
	LoaderCircleIcon,
	LockIcon,
} from '@modrinth/assets'
import { type ExtendedReport, reportQuickReplies } from '@modrinth/moderation'
import {
	Avatar,
	ButtonStyled,
	CollapsibleRegion,
	type ContentItem,
	CopyCode,
	getProjectTypeIcon,
	injectModrinthClient,
	injectNotificationManager,
	useFormatDateTime,
	useRelativeTime,
} from '@modrinth/ui'
import { formatProjectType } from '@modrinth/utils'
import { computed, ref, watch } from 'vue'

import { isStaff } from '~/helpers/users.js'

import ThreadView from '../thread/ThreadView.vue'
import SharedInstanceReportContext, {
	type SharedInstanceOwnerInstance,
	type SharedInstanceReportDetails,
	type SharedInstanceReportUser,
} from './SharedInstanceReportContext.vue'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const auth = await useAuth()

type SharedInstanceVersionDependency = Labrinth.Versions.v2.Dependency & {
	project_id?: string
	version_id?: string
}

const props = defineProps<{
	report: ExtendedReport
	sharedInstanceDetailsLoader?: () => Promise<SharedInstanceReportDetails>
	sharedInstanceVersionContentLoader?: (
		instanceId: string,
		version: number,
	) => Promise<ContentItem[]>
}>()

const reportThread = ref<{
	setReplyContent: (content: string) => void
	sendReply: (privateMessage?: boolean) => Promise<void>
} | null>(null)
const isThreadCollapsed = ref(true)
const sharedInstanceDetails = ref<SharedInstanceReportDetails | null>(null)
const sharedInstanceLoading = ref(false)
const sharedInstanceError = ref<string | null>(null)
const sharedInstanceBanPending = ref(false)
const sharedInstanceVersions = new Map<string, SharedInstances.Instances.v1.InstanceVersion>()
let sharedInstanceDetailsRequest: Promise<void> | null = null

watch(isThreadCollapsed, (collapsed) => {
	if (!collapsed) void loadSharedInstanceDetails()
})

const didCloseReport = ref(false)
const reportClosed = computed(() => {
	return didCloseReport.value || props.report.closed
})
const sharedInstanceQuarantined = computed(
	() =>
		sharedInstanceDetails.value?.quarantine ?? props.report.shared_instance?.quarantine ?? false,
)

const threadWithReportBody = computed(() => {
	if (!props.report.thread) return null

	const reportBodyMessage = {
		id: `report-body-${props.report.id}`,
		author_id: props.report.reporter_user.id,
		body: {
			type: 'text' as const,
			body: props.report.body || 'Report opened.',
			private: false,
			replying_to: null,
			associated_images: [],
		},
		created: props.report.created,
		hide_identity: false,
	}

	return {
		...props.report.thread,
		messages: [reportBodyMessage, ...props.report.thread.messages],
		members: [props.report.reporter_user, ...props.report.thread.members],
	}
})

const remainingMessageCount = computed(() => {
	if (!props.report.thread?.messages) return 0
	// Thread messages count (report body is injected separately)
	return props.report.thread.messages.length
})

const expandText = computed(() => {
	if (remainingMessageCount.value === 0) return 'Expand'
	if (remainingMessageCount.value === 1) return 'Show 1 more message'
	return `Show ${remainingMessageCount.value} more messages`
})

async function closeReport(reply = false) {
	if (reply && reportThread.value) {
		await reportThread.value.sendReply()
	}

	try {
		await useBaseFetch(`report/${props.report.id}`, {
			method: 'PATCH',
			body: {
				closed: true,
			},
		})
		await refreshReportCaches()
		didCloseReport.value = true
	} catch (err: any) {
		addNotification({
			title: 'Error closing report',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
}

async function reopenReport() {
	try {
		await useBaseFetch(`report/${props.report.id}`, {
			method: 'PATCH',
			body: {
				closed: false,
			},
		})
		await refreshReportCaches()
		didCloseReport.value = false
	} catch (err: any) {
		addNotification({
			title: 'Error reopening report',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
}

const formatRelativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})

async function refreshReportCaches() {
	await Promise.allSettled([refreshThread(), refreshNuxtData('new-moderation-reports')])
}

async function refreshThread() {
	const threadId = props.report.thread?.id ?? props.report.thread_id
	if (!threadId) return

	const thread = await useBaseFetch(`thread/${threadId}`)
	updateThread(thread)
}

function updateThread(newThread: any) {
	if (props.report.thread) {
		Object.assign(props.report.thread, newThread)
	}
}

async function getSharedInstanceVersion(
	instanceId: string,
	versionNumber: number,
): Promise<SharedInstances.Instances.v1.InstanceVersion> {
	const cacheKey = `${instanceId}:${versionNumber}`
	const cachedVersion = sharedInstanceVersions.get(cacheKey)
	if (cachedVersion) return cachedVersion

	const instanceVersion = await client.sharedinstances.instances_v1.getVersion(
		instanceId,
		versionNumber,
	)
	sharedInstanceVersions.set(cacheKey, instanceVersion)
	return instanceVersion
}

async function getOtherSharedInstances(ownerId: string): Promise<SharedInstanceOwnerInstance[]> {
	const instanceIds = await client.sharedinstances.instances_v1.getForUser(ownerId)
	const otherInstanceIds = [...new Set(instanceIds)].filter(
		(instanceId) => instanceId !== props.report.item_id,
	)
	const results = await Promise.allSettled(
		otherInstanceIds.map(async (instanceId) => {
			const [instance, instanceUsers, latestVersion] = await Promise.all([
				client.sharedinstances.instances_v1.get(instanceId),
				client.sharedinstances.instances_v1.getUsers(instanceId),
				client.sharedinstances.instances_v1.getLatestVersion(instanceId),
			])
			sharedInstanceVersions.set(`${instanceId}:${latestVersion.version}`, latestVersion)

			return {
				id: instanceId,
				name: instance.name,
				icon_url: instance.icon,
				latest_version: latestVersion.version,
				member_count: instanceUsers.users.length,
				quarantine: instance.quarantine,
			}
		}),
	)

	return results
		.flatMap((result) => (result.status === 'fulfilled' ? [result.value] : []))
		.sort((a, b) => a.name.localeCompare(b.name))
}

async function loadSharedInstanceDetails() {
	if (props.report.item_type !== 'shared-instance' || sharedInstanceDetails.value) return
	if (sharedInstanceDetailsRequest) return sharedInstanceDetailsRequest

	sharedInstanceDetailsRequest = (async () => {
		sharedInstanceLoading.value = true
		sharedInstanceError.value = null

		try {
			if (props.sharedInstanceDetailsLoader) {
				sharedInstanceDetails.value = await props.sharedInstanceDetailsLoader()
				return
			}

			const [instance, instanceUsers] = await Promise.all([
				props.report.shared_instance ??
					client.sharedinstances.instances_v1.get(props.report.item_id),
				client.sharedinstances.instances_v1.getUsers(props.report.item_id),
			])
			const userIds = [...new Set(instanceUsers.users.map((user) => user.id))]
			const users = userIds.length ? await client.labrinth.users_v2.getMultiple(userIds) : []
			const usersById = new Map(users.map((user) => [user.id, user]))
			const ownerMembership = instanceUsers.users.find((user) => user.join_type === 'owner')

			if (!ownerMembership) {
				throw new Error('The shared instance has no owner.')
			}

			const toReportUser = (
				membership: (typeof instanceUsers.users)[number],
			): SharedInstanceReportUser => {
				const user = usersById.get(membership.id)
				return {
					id: membership.id,
					username: user?.username ?? membership.id,
					avatar_url: user?.avatar_url,
					joined_at: membership.joined_at,
					last_played: membership.last_played,
					join_type: membership.join_type,
				}
			}

			const reportedVersion = props.report.shared_instance_version_id
			const versionNumbers =
				reportedVersion !== undefined
					? Array.from({ length: reportedVersion + 1 }, (_, index) => reportedVersion - index)
					: []
			const [versionDetails, otherInstancesResult] = await Promise.all([
				Promise.all(
					versionNumbers.map(async (versionNumber) => {
						try {
							const version = await getSharedInstanceVersion(props.report.item_id, versionNumber)
							return {
								version: version.version,
								game_version: version.game_version,
								loader: version.loader,
								loader_version: version.loader_version,
							}
						} catch {
							return { version: versionNumber }
						}
					}),
				),
				getOtherSharedInstances(ownerMembership.id)
					.then((instances) => ({ instances, loaded: true }))
					.catch(() => ({ instances: [], loaded: false })),
			])

			sharedInstanceDetails.value = {
				id: props.report.item_id,
				name: instance.name,
				icon_url: instance.icon,
				quarantine: instance.quarantine,
				owner: toReportUser(ownerMembership),
				members: instanceUsers.users
					.filter((user) => user.id !== ownerMembership.id)
					.map(toReportUser),
				reported_version: versionDetails[0],
				previous_versions: versionDetails.slice(1),
				other_instances: otherInstancesResult.instances,
				other_instances_loaded: otherInstancesResult.loaded,
			}
		} catch (error) {
			sharedInstanceError.value = getErrorMessage(error, 'Failed to load shared instance details.')
		} finally {
			sharedInstanceLoading.value = false
			sharedInstanceDetailsRequest = null
		}
	})()

	return sharedInstanceDetailsRequest
}

async function loadSharedInstanceVersionContent(
	instanceId: string,
	versionNumber: number,
): Promise<ContentItem[]> {
	if (props.report.item_type !== 'shared-instance') return []
	if (props.sharedInstanceVersionContentLoader) {
		return props.sharedInstanceVersionContentLoader(instanceId, versionNumber)
	}

	const instanceVersion = await getSharedInstanceVersion(instanceId, versionNumber)

	const modpackVersionId = instanceVersion.modpack_id
	const directVersionIds = (instanceVersion.modrinth_ids ?? []).filter(
		(versionId) => versionId !== modpackVersionId,
	)
	const modpackVersion = modpackVersionId
		? await client.labrinth.versions_v2.getVersion(modpackVersionId)
		: null
	const modpackDependencies = (modpackVersion?.dependencies ??
		[]) as SharedInstanceVersionDependency[]
	const dependencyVersionIds = modpackDependencies.flatMap((dependency) =>
		dependency.version_id ? [dependency.version_id] : [],
	)
	const uniqueVersionIds = [...new Set([...directVersionIds, ...dependencyVersionIds])]
	const versions = uniqueVersionIds.length
		? await client.labrinth.versions_v2.getVersions(uniqueVersionIds)
		: []
	const dependencyProjectIds = modpackDependencies.flatMap((dependency) =>
		dependency.project_id ? [dependency.project_id] : [],
	)
	const projectIds = [
		...new Set([...versions.map((version) => version.project_id), ...dependencyProjectIds]),
	]
	const projects = projectIds.length
		? await client.labrinth.projects_v2.getMultiple(projectIds)
		: []
	const versionsById = new Map(versions.map((version) => [version.id, version]))
	const projectsById = new Map(projects.map((project) => [project.id, project]))

	const directContent: ContentItem[] = [...new Set(directVersionIds)].flatMap((versionId) => {
		const version = versionsById.get(versionId)
		if (!version) return []

		const project = projectsById.get(version.project_id)
		return [sharedInstanceContentItem(version, project)]
	})

	const modpackContent: ContentItem[] = modpackDependencies.map((dependency) => {
		const version = dependency.version_id ? versionsById.get(dependency.version_id) : undefined
		const project = dependency.project_id
			? projectsById.get(dependency.project_id)
			: version
				? projectsById.get(version.project_id)
				: undefined
		const primaryFile = version
			? (version.files.find((file) => file.primary) ?? version.files[0])
			: undefined
		const fileName =
			primaryFile?.filename ?? dependency.file_name ?? project?.title ?? version?.name ?? 'Unknown'

		return sharedInstanceContentItem(
			version,
			project,
			fileName,
			dependency.project_id ?? fileName,
			!project && !version,
		)
	})

	const externalContent: ContentItem[] = instanceVersion.external_files.map((file, index) => ({
		id: `external:${file.file_type}:${file.file_name}:${index}`,
		file_name: file.file_name,
		size: file.file_size,
		project_type: file.file_type,
		has_update: false,
		update_version_id: null,
		source_kind: 'shared_instance',
		external: true,
		external_url: file.url,
		project: {
			id: file.file_name,
			slug: file.file_name,
			title: file.file_name,
			icon_url: undefined,
		},
	}))

	return [...externalContent, ...modpackContent, ...directContent]
}

function sharedInstanceContentItem(
	version: Labrinth.Versions.v2.Version | undefined,
	project: Labrinth.Projects.v2.Project | undefined,
	fallbackFileName?: string,
	fallbackProjectId = version?.project_id ?? fallbackFileName ?? 'unknown',
	external = false,
): ContentItem {
	const primaryFile = version
		? (version.files.find((file) => file.primary) ?? version.files[0])
		: undefined
	const fileName =
		primaryFile?.filename ?? fallbackFileName ?? project?.title ?? version?.name ?? 'Unknown'

	return {
		id: version?.id ?? project?.id ?? fileName,
		file_name: fileName,
		size: primaryFile?.size,
		project_type: project?.project_type ?? 'mod',
		has_update: false,
		update_version_id: null,
		source_kind: 'shared_instance',
		external,
		project: {
			id: project?.id ?? fallbackProjectId,
			slug: project?.slug ?? fallbackProjectId,
			title: project?.title ?? version?.name ?? fileName,
			icon_url: project?.icon_url ?? undefined,
		},
		...(version
			? {
					version: {
						id: version.id,
						version_number: version.version_number,
						file_name: fileName,
						date_published: version.date_published,
					},
				}
			: {}),
	}
}

function showSharedInstanceContentError(error: unknown) {
	addNotification({
		type: 'error',
		title: 'Failed to load version content',
		text: getErrorMessage(
			error,
			'The content for this shared instance version could not be loaded.',
		),
	})
}

function getErrorMessage(error: unknown, fallback: string) {
	if (typeof error === 'string') return error
	if (!error || typeof error !== 'object') return fallback

	const requestError = error as {
		message?: string
		data?: {
			description?: string
		}
	}
	return requestError.data?.description ?? requestError.message ?? fallback
}

const reportItemAvatarUrl = computed(() => {
	switch (props.report.item_type) {
		case 'project':
		case 'version':
			return props.report.project?.icon_url || ''
		case 'user':
			return props.report.user?.avatar_url || ''
		case 'shared-instance':
			return sharedInstanceDetails.value?.icon_url || props.report.shared_instance?.icon || ''
		default:
			return undefined
	}
})

const reportItemTitle = computed(() => {
	if (props.report.item_type === 'user') return props.report.user?.username || 'Unknown User'
	if (props.report.item_type === 'shared-instance') {
		return (
			sharedInstanceDetails.value?.name || props.report.shared_instance?.name || 'Shared instance'
		)
	}

	return props.report.project?.title || 'Unknown Project'
})

const reportItemUrl = computed(() => {
	switch (props.report.item_type) {
		case 'user':
			return `/user/${props.report.user?.username}`
		case 'project':
			return `/${props.report.project?.project_type}/${props.report.project?.slug}`
		case 'version':
			return `/${props.report.project?.project_type}/${props.report.project?.slug}/version/${props.report.version?.id}`
		case 'shared-instance':
			return ''
		default:
			return `/${props.report.item_type}/${props.report.id}`
	}
})

const formattedReportType = computed(() => {
	const reportType = props.report.report_type

	// some are split by -, some are split by " "
	const words = reportType.includes('-') ? reportType.split('-') : reportType.split(' ')
	return words.map((word) => word.charAt(0).toUpperCase() + word.slice(1)).join(' ')
})

function copyId() {
	navigator.clipboard.writeText(props.report.id).then(() => {
		addNotification({
			type: 'success',
			title: 'Report ID copied',
			text: 'The ID of this report has been copied to your clipboard.',
		})
	})
}

async function banSharedInstanceOwner(owner: SharedInstanceReportUser) {
	if (sharedInstanceBanPending.value) return
	sharedInstanceBanPending.value = true

	try {
		await client.sharedinstances.moderation_v1.blacklistUsers({
			user_ids: [owner.id],
		})

		if (sharedInstanceDetails.value) {
			sharedInstanceDetails.value = {
				...sharedInstanceDetails.value,
				quarantine: true,
				other_instances: sharedInstanceDetails.value.other_instances.map((instance) => ({
					...instance,
					quarantine: true,
				})),
			}
		}

		addNotification({
			type: 'success',
			title: 'Owner banned from shared instances',
			text: `${owner.username} has been banned and all of their shared instances have been quarantined.`,
		})
	} catch (error) {
		addNotification({
			type: 'error',
			title: 'Failed to ban shared instance owner',
			text: getErrorMessage(error, `Could not ban ${owner.username} from shared instances.`),
		})
	} finally {
		sharedInstanceBanPending.value = false
	}
}
</script>
