<template>
	<NewModal
		ref="contextModal"
		:header="
			selectedInstance ? `${selectedInstance.name} moderation context` : 'Shared instance context'
		"
		no-padding
		scrollable
		max-width="90rem"
		width="calc(100vw - 2rem)"
		max-content-height="85vh"
	>
		<div
			v-if="selectedContextPending"
			class="flex min-h-64 items-center justify-center gap-2 text-secondary"
		>
			<LoaderCircleIcon class="size-6 animate-spin" />
			Loading moderation context…
		</div>
		<div v-else-if="selectedContextError" class="p-6">
			<Admonition
				type="critical"
				header="Failed to load moderation context"
				:body="selectedContextErrorMessage"
				show-actions-underneath
			>
				<template #actions>
					<Button :disabled="selectedContextFetching" @click="refetchSelectedContext()">
						<LoaderCircleIcon v-if="selectedContextFetching" class="animate-spin" />
						Try again
					</Button>
				</template>
			</Admonition>
		</div>
		<SharedInstanceReportContext
			v-else-if="selectedContext"
			:key="selectedContext.id"
			:details="selectedContext"
			:ban-pending="banOwnerPending"
			:load-version-content="sharedInstanceContent.loadVersionContent"
			context-type="moderation"
			class="!border-t-0"
			@ban-owner="banSharedInstanceOwner"
			@content-error="showSharedInstanceContentError"
		/>
	</NewModal>

	<div class="normal-page no-sidebar !pb-6">
		<div class="normal-page__content flex flex-col gap-4">
			<header
				class="flex flex-col gap-3 border-0 border-b border-solid border-divider pb-4 sm:flex-row sm:items-center sm:justify-between"
			>
				<div class="flex min-w-0 items-center gap-3">
					<Avatar
						:src="user?.avatar_url"
						:alt="user?.username ?? userId"
						:tint-by="user?.id ?? userId"
						size="48px"
						circle
					/>
					<div class="min-w-0">
						<h1 class="m-0 truncate text-2xl font-extrabold text-contrast">
							{{ user?.username ?? userId }}'s shared instances
						</h1>
						<p class="m-0 text-secondary">All shared instances this user owns or belongs to.</p>
					</div>
				</div>

				<ButtonLink :to="`/user/${user?.id ?? userId}`" target="_blank" class="w-fit">
					<UserIcon />
					User profile
					<ExternalIcon class="size-4" />
				</ButtonLink>
			</header>

			<div
				v-if="isPending"
				class="flex min-h-48 items-center justify-center gap-2 rounded-2xl border border-solid border-surface-4 bg-surface-3 text-secondary"
			>
				<LoaderCircleIcon class="size-6 animate-spin" />
				Loading shared instances…
			</div>

			<Admonition
				v-else-if="error"
				type="critical"
				header="Failed to load shared instances"
				:body="errorMessage"
				show-actions-underneath
			>
				<template #actions>
					<Button :disabled="isFetching" @click="retry">
						<LoaderCircleIcon v-if="isFetching" class="animate-spin" />
						Try again
					</Button>
				</template>
			</Admonition>

			<template v-else>
				<Admonition
					v-if="sharedInstances?.unavailableCount"
					type="warning"
					header="Some shared instances are unavailable"
					:body="`${sharedInstances.unavailableCount} ${sharedInstances.unavailableCount === 1 ? 'instance could' : 'instances could'} not be loaded.`"
				/>

				<div
					v-if="!sharedInstances?.instances.length"
					class="flex min-h-48 flex-col items-center justify-center gap-2 rounded-2xl border border-solid border-surface-4 bg-surface-3 p-6 text-center"
				>
					<BoxesIcon class="size-10 text-secondary" />
					<h2 class="m-0 text-lg font-semibold text-contrast">No shared instances</h2>
					<p class="m-0 text-secondary">
						This user does not own or belong to any shared instances.
					</p>
				</div>

				<div v-else class="grid gap-3 md:grid-cols-2">
					<article
						v-for="instance in sharedInstances.instances"
						:key="instance.id"
						class="flex min-w-0 flex-col gap-4 rounded-2xl border border-solid border-surface-4 bg-surface-3 p-4"
					>
						<div class="flex min-w-0 items-center gap-3">
							<Avatar
								:src="instance.iconUrl"
								:alt="instance.name"
								:tint-by="instance.id"
								size="48px"
								no-shadow
							/>
							<div class="min-w-0 flex-1">
								<div class="flex min-w-0 items-center gap-2">
									<h2 class="m-0 truncate text-lg font-semibold text-contrast">
										{{ instance.name }}
									</h2>
									<span
										v-if="instance.quarantine"
										class="bg-orange-highlight inline-flex shrink-0 items-center gap-1 rounded-full px-2 py-0.5 text-xs font-semibold text-orange"
									>
										<LockIcon class="size-3.5" />
										Quarantined
									</span>
								</div>
								<span class="text-sm text-secondary">{{ relationLabel(instance.membership) }}</span>
							</div>
						</div>

						<div class="grid gap-2 text-sm text-primary sm:grid-cols-2">
							<div class="flex min-w-0 items-center gap-2">
								<VersionIcon class="size-5 shrink-0 text-secondary" />
								<span class="truncate">
									<template v-if="instance.latestVersion">
										Version {{ instance.latestVersion.version }} ·
										{{ formattedVersion(instance.latestVersion) }}
									</template>
									<template v-else>Version unavailable</template>
								</span>
							</div>
							<div class="flex items-center gap-2">
								<UsersIcon class="size-5 shrink-0 text-secondary" />
								{{ instance.memberCount }}
								{{ instance.memberCount === 1 ? 'member' : 'members' }}
							</div>
							<div class="flex items-center gap-2">
								<HistoryIcon class="size-5 shrink-0 text-secondary" />
								<span
									v-if="instance.membership?.last_played"
									v-tooltip="formatDateTime(instance.membership.last_played)"
								>
									Last played {{ formatRelativeTime(instance.membership.last_played) }}
								</span>
								<span v-else>Never played</span>
							</div>
							<div class="flex items-center gap-2">
								<CalendarIcon class="size-5 shrink-0 text-secondary" />
								<span
									v-if="instance.membership?.joined_at"
									v-tooltip="formatDateTime(instance.membership.joined_at)"
								>
									Joined {{ formatRelativeTime(instance.membership.joined_at) }}
								</span>
								<span v-else>Join date unavailable</span>
							</div>
						</div>

						<div class="mt-auto flex flex-col gap-1.5">
							<span class="text-xs font-semibold uppercase tracking-wide text-secondary">
								Instance ID
							</span>
							<CopyCode :text="instance.id" />
						</div>
						<Button
							type="colored"
							color="orange"
							class="w-full"
							@click="openInstanceContext(instance)"
						>
							<ScaleIcon />
							View context
						</Button>
					</article>
				</div>
			</template>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { SharedInstances } from '@modrinth/api-client'
import {
	BoxesIcon,
	CalendarIcon,
	ExternalIcon,
	HistoryIcon,
	LoaderCircleIcon,
	LockIcon,
	ScaleIcon,
	UserIcon,
	UsersIcon,
	VersionIcon,
} from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	Button,
	ButtonLink,
	CopyCode,
	injectModrinthClient,
	injectNotificationManager,
	NewModal,
	useFormatDateTime,
	useRelativeTime,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, onServerPrefetch, ref } from 'vue'

import SharedInstanceReportContext, {
	type SharedInstanceOwnerInstance,
	type SharedInstanceReportDetails,
	type SharedInstanceReportUser,
} from '~/components/ui/moderation/SharedInstanceReportContext.vue'
import { createSharedInstanceContentLoader } from '~/helpers/shared-instance-content'

type InstanceMembership = SharedInstances.Instances.v1.InstanceUser
type InstanceVersion = SharedInstances.Instances.v1.InstanceVersion
type UserSharedInstance = {
	id: string
	name: string
	iconUrl: string | null
	quarantine: boolean
	memberCount: number
	membership?: InstanceMembership
	latestVersion: InstanceVersion | null
}

const route = useRoute()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const userId = computed(() => String(route.params.user ?? ''))
const canLoadUser = computed(() => userId.value.length > 0)
const selectedInstance = ref<UserSharedInstance | null>(null)
const contextModal = ref<InstanceType<typeof NewModal> | null>(null)
const sharedInstanceContent = createSharedInstanceContentLoader(client)
const formatRelativeTime = useRelativeTime({ style: 'narrow' })
const formatDateTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })

const {
	data: user,
	error: userError,
	isPending: userPending,
	isFetching: userFetching,
	refetch: refetchUser,
	suspense: userSuspense,
} = useQuery({
	queryKey: computed(() => ['user', 'v3', userId.value] as const),
	queryFn: () => client.labrinth.users_v3.get(userId.value),
	enabled: canLoadUser,
})

const {
	data: sharedInstances,
	error: sharedInstancesError,
	isPending: sharedInstancesPending,
	isFetching: sharedInstancesFetching,
	refetch: refetchSharedInstances,
	suspense: sharedInstancesSuspense,
} = useQuery({
	queryKey: computed(() => ['shared-instances', 'user', userId.value] as const),
	queryFn: loadSharedInstances,
	enabled: canLoadUser,
})

const selectedInstanceId = computed(() => selectedInstance.value?.id ?? null)
const {
	data: selectedContext,
	error: selectedContextError,
	isPending: selectedContextPending,
	isFetching: selectedContextFetching,
	refetch: refetchSelectedContext,
} = useQuery({
	queryKey: computed(() => sharedInstanceContextQueryKey(selectedInstanceId.value)),
	queryFn: () => loadSharedInstanceContext(selectedInstanceId.value),
	enabled: computed(() => selectedInstanceId.value !== null),
	staleTime: 30_000,
})

const banOwnerMutation = useMutation({
	mutationFn: (owner: SharedInstanceReportUser) =>
		client.sharedinstances.moderation_v1.blacklistUsers({ user_ids: [owner.id] }),
	onSuccess: async (_data, owner) => {
		const instanceId = selectedInstanceId.value
		if (instanceId) {
			queryClient.setQueryData<SharedInstanceReportDetails>(
				sharedInstanceContextQueryKey(instanceId),
				(details) =>
					details
						? {
								...details,
								quarantine: true,
								other_instances: details.other_instances.map((instance) => ({
									...instance,
									quarantine: true,
								})),
							}
						: details,
			)
		}

		addNotification({
			type: 'success',
			title: 'Owner banned from shared instances',
			text: `${owner.username} has been banned and all of their shared instances have been quarantined.`,
		})

		await Promise.all([
			queryClient.invalidateQueries({ queryKey: ['shared-instances', 'user', userId.value] }),
			queryClient.invalidateQueries({
				queryKey: ['shared-instance-blacklist', 'v1', owner.id],
			}),
		])
	},
	onError: (requestError, owner) => {
		addNotification({
			type: 'error',
			title: 'Failed to ban shared instance owner',
			text: getErrorMessage(requestError, `Could not ban ${owner.username} from shared instances.`),
		})
	},
})

const isPending = computed(() => userPending.value || sharedInstancesPending.value)
const isFetching = computed(() => userFetching.value || sharedInstancesFetching.value)
const error = computed(() => userError.value ?? sharedInstancesError.value)
const errorMessage = computed(() =>
	getErrorMessage(error.value, 'The shared instances service could not be reached.'),
)
const selectedContextErrorMessage = computed(() =>
	getErrorMessage(selectedContextError.value, 'The moderation context could not be loaded.'),
)
const banOwnerPending = computed(() => banOwnerMutation.isPending.value)

useHead({
	title: computed(() => `${user.value?.username ?? userId.value}'s shared instances - Modrinth`),
})

onServerPrefetch(() => Promise.all([userSuspense(), sharedInstancesSuspense()]))

async function loadSharedInstances(): Promise<{
	instances: UserSharedInstance[]
	unavailableCount: number
}> {
	const currentUserId = userId.value
	const instanceIds = [
		...new Set(await client.sharedinstances.instances_v1.getForUser(currentUserId)),
	]
	const results = await Promise.allSettled(
		instanceIds.map(async (instanceId): Promise<UserSharedInstance> => {
			const [instance, instanceUsers, latestVersion] = await Promise.all([
				client.sharedinstances.instances_v1.get(instanceId),
				client.sharedinstances.instances_v1.getUsers(instanceId),
				client.sharedinstances.instances_v1.getLatestVersion(instanceId).catch(() => null),
			])

			return {
				id: instanceId,
				name: instance.name,
				iconUrl: instance.icon,
				quarantine: instance.quarantine,
				memberCount: instanceUsers.users.length,
				membership: instanceUsers.users.find((membership) => membership.id === currentUserId),
				latestVersion,
			}
		}),
	)
	const instances = results
		.flatMap((result) => (result.status === 'fulfilled' ? [result.value] : []))
		.sort((first, second) => first.name.localeCompare(second.name))
	const unavailableCount = results.length - instances.length

	if (instanceIds.length > 0 && instances.length === 0) {
		throw new Error("None of this user's shared instances could be loaded.")
	}

	return { instances, unavailableCount }
}

function sharedInstanceContextQueryKey(instanceId: string | null) {
	return ['shared-instance', 'moderation-context', 'v1', instanceId] as const
}

function openInstanceContext(instance: UserSharedInstance) {
	selectedInstance.value = instance
	contextModal.value?.show()
}

async function loadSharedInstanceContext(
	instanceId: string | null,
): Promise<SharedInstanceReportDetails> {
	if (!instanceId) throw new Error('No shared instance was selected.')

	const [instance, instanceUsers, latestVersion] = await Promise.all([
		client.sharedinstances.instances_v1.get(instanceId),
		client.sharedinstances.instances_v1.getUsers(instanceId),
		client.sharedinstances.instances_v1.getLatestVersion(instanceId),
	])
	sharedInstanceContent.cacheVersion(instanceId, latestVersion)

	const memberIds = [...new Set(instanceUsers.users.map((membership) => membership.id))]
	const members = memberIds.length ? await client.labrinth.users_v2.getMultiple(memberIds) : []
	const membersById = new Map(members.map((member) => [member.id, member]))
	const ownerMembership = instanceUsers.users.find((membership) => membership.join_type === 'owner')
	if (!ownerMembership) throw new Error('The shared instance has no owner.')

	const toContextUser = (membership: InstanceMembership): SharedInstanceReportUser => {
		const member = membersById.get(membership.id)
		return {
			id: membership.id,
			username: member?.username ?? membership.id,
			avatar_url: member?.avatar_url,
			joined_at: membership.joined_at,
			last_played: membership.last_played,
			join_type: membership.join_type,
		}
	}

	const versionNumbers = Array.from(
		{ length: latestVersion.version + 1 },
		(_value, index) => latestVersion.version - index,
	)
	const [versionDetails, otherInstancesResult] = await Promise.all([
		Promise.all(
			versionNumbers.map(async (versionNumber) => {
				try {
					const version = await sharedInstanceContent.getVersion(instanceId, versionNumber)
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
		loadOtherSharedInstances(ownerMembership.id, instanceId)
			.then((instances) => ({ instances, loaded: true }))
			.catch(() => ({ instances: [], loaded: false })),
	])

	return {
		id: instanceId,
		name: instance.name,
		icon_url: instance.icon,
		quarantine: instance.quarantine,
		owner: toContextUser(ownerMembership),
		members: instanceUsers.users
			.filter((membership) => membership.id !== ownerMembership.id)
			.map(toContextUser),
		reported_version: versionDetails[0],
		previous_versions: versionDetails.slice(1),
		other_instances: otherInstancesResult.instances,
		other_instances_loaded: otherInstancesResult.loaded,
	}
}

async function loadOtherSharedInstances(
	ownerId: string,
	selectedInstanceId: string,
): Promise<SharedInstanceOwnerInstance[]> {
	const instanceIds = await client.sharedinstances.instances_v1.getForUser(ownerId)
	const otherInstanceIds = [...new Set(instanceIds)].filter(
		(instanceId) => instanceId !== selectedInstanceId,
	)
	const results = await Promise.allSettled(
		otherInstanceIds.map(async (instanceId): Promise<SharedInstanceOwnerInstance> => {
			const [instance, instanceUsers, latestVersion] = await Promise.all([
				client.sharedinstances.instances_v1.get(instanceId),
				client.sharedinstances.instances_v1.getUsers(instanceId),
				client.sharedinstances.instances_v1.getLatestVersion(instanceId),
			])
			sharedInstanceContent.cacheVersion(instanceId, latestVersion)

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
		.sort((first, second) => first.name.localeCompare(second.name))
}

function banSharedInstanceOwner(owner: SharedInstanceReportUser) {
	banOwnerMutation.mutate(owner)
}

function showSharedInstanceContentError(requestError: unknown) {
	addNotification({
		type: 'error',
		title: 'Failed to load version content',
		text: getErrorMessage(
			requestError,
			'The content for this shared instance version could not be loaded.',
		),
	})
}

function relationLabel(membership?: InstanceMembership): string {
	if (!membership) return 'Membership unavailable'
	if (membership.join_type === 'owner') return 'Owner'
	if (membership.join_type === 'link') return 'Joined via share link'
	return 'Joined via direct invite'
}

function formattedVersion(version: InstanceVersion): string {
	const loader = version.loader
		? version.loader.charAt(0).toUpperCase() + version.loader.slice(1)
		: 'Vanilla'
	const formattedLoader = version.loader_version ? `${loader} ${version.loader_version}` : loader
	return `Minecraft ${version.game_version} · ${formattedLoader}`
}

function getErrorMessage(requestError: unknown, fallback: string): string {
	if (typeof requestError === 'string') return requestError
	if (!requestError || typeof requestError !== 'object') return fallback

	const typedError = requestError as {
		message?: string
		data?: { description?: string }
	}
	return typedError.data?.description ?? typedError.message ?? fallback
}

function retry() {
	void Promise.all([refetchUser(), refetchSharedInstances()])
}
</script>
