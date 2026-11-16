<script setup lang="ts">
import {
	BanIcon,
	BoxesIcon,
	BoxIcon,
	EditIcon,
	EyeIcon,
	LinkIcon,
	LockIcon,
	LockOpenIcon,
	MoreHorizontalIcon,
	TriangleAlertIcon,
} from '@modrinth/assets'
import {
	Admonition,
	Button,
	ButtonLink,
	type ButtonMenuOption,
	injectModrinthClient,
	TeleportOverflowMenu,
} from '@modrinth/ui'
import { renderString } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import {
	type SharedInstanceModerationDetails,
	sharedInstanceQueryOptions,
} from '~/composables/queries/shared-instance'
import { useCopyText, useSharedInstanceModeration } from '~/composables/shared-instance-moderation'
import {
	formatSharedInstanceLoader,
	SHARED_INSTANCE_CONFIG_FILE_TYPES,
	sharedInstanceInspectorUrl,
} from '~/helpers/shared-instance-moderation'

import SharedInstanceInspectorModal from './shared-instance-inspector/index.vue'
import SharedInstanceBanModal from './SharedInstanceBanModal.vue'
import SharedInstanceQuarantineModal from './SharedInstanceQuarantineModal.vue'

const CONTENT_TYPE_LABELS: Record<string, [string, string]> = {
	mod: ['mod', 'mods'],
	resourcepack: ['resource pack', 'resource packs'],
	shader: ['shader', 'shaders'],
	shaderpack: ['shader', 'shaders'],
	datapack: ['data pack', 'data packs'],
	plugin: ['plugin', 'plugins'],
}

const props = defineProps<{
	details: SharedInstanceModerationDetails
	reportedVersion?: number
}>()

const client = injectModrinthClient()
const queryClient = useQueryClient()
const copyText = useCopyText()
const { pending, liftQuarantine, banUser, unbanUser } = useSharedInstanceModeration()

const inspectorModal = ref<InstanceType<typeof SharedInstanceInspectorModal> | null>(null)
const quarantineModal = ref<InstanceType<typeof SharedInstanceQuarantineModal> | null>(null)
const banModal = ref<InstanceType<typeof SharedInstanceBanModal> | null>(null)

const contextVersion = computed(
	() =>
		props.details.versions.find((version) => version.version === props.reportedVersion) ??
		props.details.versions[0],
)
const instanceSummary = computed(() => ({
	id: props.details.id,
	name: props.details.name,
	icon_url: props.details.icon_url,
}))
const ownerSummary = computed(() => ({
	id: props.details.owner.id,
	username: props.details.owner.username,
}))
const instanceQuarantine = computed(() =>
	props.details.quarantine_details?.scope === 'instance' ? props.details.quarantine_details : null,
)
const renderedReason = computed(() => renderString(instanceQuarantine.value?.reason ?? ''))

const { data: ownerBlacklist } = useQuery(
	computed(() => sharedInstanceQueryOptions.blacklist(props.details.owner.id, client)),
)
const ownerBanned = computed(() => ownerBlacklist.value?.blacklisted === true)

const { data: contextVersionDetails } = useQuery(
	computed(() => ({
		...sharedInstanceQueryOptions.version(
			props.details.id,
			contextVersion.value?.version ?? 0,
			client,
		),
		enabled: contextVersion.value !== undefined,
	})),
)

const {
	data: content,
	error: contentError,
	isFetching: contentFetching,
	isPending: contentQueryPending,
	refetch: refetchContent,
} = useQuery(
	computed(() => ({
		...sharedInstanceQueryOptions.versionContent(
			props.details.id,
			contextVersion.value?.version ?? 0,
			client,
			queryClient,
		),
		enabled: contextVersion.value !== undefined,
	})),
)

const contentFailed = computed(() => contentError.value !== null && content.value === undefined)
const contentPending = computed(
	() => contextVersion.value !== undefined && contentQueryPending.value && !contentFailed.value,
)

const quarantinedById = computed(() => instanceQuarantine.value?.quarantined_by ?? null)
const { data: quarantinedBy } = useQuery(
	computed(() => ({
		...sharedInstanceQueryOptions.moderator(quarantinedById.value ?? '', client),
		enabled: quarantinedById.value !== null,
	})),
)

const bannedById = computed(() =>
	ownerBanned.value ? (ownerBlacklist.value?.blacklisted_by ?? null) : null,
)
const { data: bannedBy } = useQuery(
	computed(() => ({
		...sharedInstanceQueryOptions.moderator(bannedById.value ?? '', client),
		enabled: bannedById.value !== null,
	})),
)

const uploadedFileCount = computed(
	() =>
		content.value?.filter(
			(item) => item.external && !SHARED_INSTANCE_CONFIG_FILE_TYPES.has(item.project_type),
		).length ?? 0,
)

const contentSummary = computed(() => {
	const counts = new Map<string, { singular: string; plural: string; count: number }>()
	for (const item of content.value ?? []) {
		if (SHARED_INSTANCE_CONFIG_FILE_TYPES.has(item.project_type)) continue
		const [singular, plural] = CONTENT_TYPE_LABELS[item.project_type] ?? [
			item.project_type,
			item.project_type,
		]
		const count = (counts.get(plural)?.count ?? 0) + 1
		counts.set(plural, { singular, plural, count })
	}
	if (counts.size === 0) return 'No content'
	return [...counts.values()]
		.sort((first, second) => second.count - first.count)
		.map(({ singular, plural, count }) => `${count} ${count === 1 ? singular : plural}`)
		.join(' · ')
})

const memberSummary = computed(() => {
	const players = [props.details.owner, ...props.details.members]
	const played = players.filter((member) => member.last_played).length
	const playedText =
		played === 0
			? 'nobody has played'
			: played === players.length
				? 'everyone has played'
				: `${played} played`
	return `${players.length} · ${playedText}`
})

const ownerSummaryText = computed(() => {
	const count = props.details.owner_instance_count
	const instances =
		count === null ? 'Unknown instances' : `${count} ${count === 1 ? 'instance' : 'instances'}`
	if (!ownerBlacklist.value) return instances
	return `${instances} · ${ownerBanned.value ? 'banned' : 'not banned'}`
})

const quarantineHeader = computed(() =>
	quarantinedBy.value
		? `Instance quarantined by ${quarantinedBy.value.username}`
		: 'Instance quarantined',
)

const quarantineNotice = computed(() => {
	const quarantine = instanceQuarantine.value
	if (!quarantine) return ''
	if (quarantine.reason) {
		if (quarantine.owner_notified === true) return 'The owner was notified with a message:'
		if (quarantine.owner_notified === false) return "The owner wasn't notified. Quarantine message:"
		return 'Quarantine message:'
	}
	if (quarantine.owner_notified === true)
		return 'Quarantined without a message. The owner was notified.'
	if (quarantine.owner_notified === false)
		return "Quarantined without a message. The owner wasn't notified."
	return 'Quarantined without a message.'
})

const banHeader = computed(() =>
	bannedBy.value
		? `${props.details.owner.username} was banned from shared instances by ${bannedBy.value.username}`
		: `${props.details.owner.username} is banned from shared instances`,
)

const ownerInstancesPath = computed(() => `/admin/shared-instances/${props.details.owner.id}`)

const moreOptions = computed<ButtonMenuOption[]>(() => [
	{
		id: 'owner-instances',
		type: 'link',
		label: "View owner's instances",
		icon: BoxIcon,
		to: ownerInstancesPath.value,
	},
	{
		id: 'ban-owner',
		label: 'Ban from shared instances',
		icon: BanIcon,
		tone: 'red',
		shown: !ownerBanned.value,
		action: promptBanOwner,
	},
	{ type: 'divider' },
	{
		id: 'copy-inspector-link',
		label: 'Copy inspector link',
		icon: LinkIcon,
		action: copyInspectorLink,
	},
])

function openInspector() {
	inspectorModal.value?.show(props.details.id, {
		version: contextVersion.value?.version,
		reportedVersions: props.reportedVersion !== undefined ? [props.reportedVersion] : [],
	})
}

async function promptBanOwner() {
	if (!(await banModal.value?.prompt())) return
	await banUser(ownerSummary.value)
}

function copyInspectorLink() {
	return copyText(
		sharedInstanceInspectorUrl(
			props.details.owner.id,
			props.details.id,
			contextVersion.value?.version,
		),
		'Inspector link copied',
	)
}

defineExpose({ openInspector })
</script>

<template>
	<section
		aria-label="Shared instance"
		class="grid grid-cols-[2rem_minmax(0,1fr)] gap-x-2 gap-y-2 border-0 border-t border-solid border-surface-4 px-4 py-3"
	>
		<div
			class="backed-svg circle raised border border-solid border-surface-5 shadow-[var(--shadow-card)]"
			style="--size: 2rem"
		>
			<BoxesIcon />
		</div>
		<div class="mt-1.5 flex flex-wrap items-center gap-2">
			<span class="font-bold text-contrast">Shared instance</span>
			<span v-if="contextVersion" class="text-sm text-secondary">
				Version {{ contextVersion.version }}
				<template v-if="contextVersionDetails?.game_version">
					· Minecraft {{ contextVersionDetails.game_version }} ·
					{{ formatSharedInstanceLoader(contextVersionDetails) }}
				</template>
			</span>
		</div>

		<div class="col-start-2 flex flex-col gap-3">
			<Admonition
				v-if="instanceQuarantine"
				type="warning"
				:header="quarantineHeader"
				:timestamp="instanceQuarantine.quarantined_at ?? undefined"
			>
				<template #icon="{ iconClass }">
					<LockIcon :class="iconClass" />
				</template>
				<span>{{ quarantineNotice }}</span>
				<!-- eslint-disable-next-line vue/no-v-html -->
				<div
					v-if="instanceQuarantine.reason"
					class="markdown-body text-contrast/85 mt-2 break-words rounded-xl bg-black/20 px-4 py-3 [&>:first-child]:mt-0 [&>:last-child]:mb-0"
					v-html="renderedReason"
				/>
				<template #actions>
					<div class="mt-2 flex flex-wrap gap-2">
						<Button
							type="colored"
							color="orange"
							:loading="pending"
							@click="liftQuarantine(instanceSummary)"
						>
							<LockOpenIcon />
							Lift quarantine
						</Button>
						<Button
							type="outlined"
							color="orange"
							@click="quarantineModal?.show({ reason: instanceQuarantine.reason })"
						>
							<EditIcon />
							{{ instanceQuarantine.reason ? 'Edit message' : 'Add message' }}
						</Button>
						<Button type="outlined" color="orange" @click="openInspector">
							<EyeIcon />
							Inspect instance
						</Button>
					</div>
				</template>
			</Admonition>

			<Admonition
				v-if="ownerBanned"
				type="critical"
				:header="banHeader"
				:timestamp="ownerBlacklist?.blacklisted_at ?? undefined"
				body="All of their shared instances are quarantined, and members can't launch them."
			>
				<template #actions>
					<div class="mt-2 flex flex-wrap gap-2">
						<Button type="colored" color="red" :loading="pending" @click="unbanUser(ownerSummary)">
							Unban owner
						</Button>
						<ButtonLink :to="ownerInstancesPath" type="outlined" color="red">
							View their instances
						</ButtonLink>
					</div>
				</template>
			</Admonition>

			<div
				v-if="!instanceQuarantine && !ownerBanned"
				class="overflow-hidden rounded-2xl border border-solid border-surface-4 bg-surface-3"
			>
				<dl class="m-0 grid grid-cols-2 lg:grid-cols-4">
					<div class="flex min-w-0 flex-col gap-0.5 px-4 py-3">
						<dt class="text-sm font-semibold text-secondary">Uploaded files</dt>
						<dd
							class="m-0 flex items-center gap-1.5 font-semibold"
							:class="uploadedFileCount > 0 ? 'text-orange' : 'text-contrast'"
						>
							<template v-if="contentPending">Loading…</template>
							<template v-else-if="contentFailed">Failed to load</template>
							<template v-else-if="uploadedFileCount > 0">
								<TriangleAlertIcon class="size-4 shrink-0" />
								{{ uploadedFileCount }} not on Modrinth
							</template>
							<template v-else>None</template>
						</dd>
					</div>
					<div
						class="flex min-w-0 flex-col gap-0.5 border-0 border-l border-solid border-surface-4 px-4 py-3"
					>
						<dt class="text-sm font-semibold text-secondary">Content</dt>
						<dd class="m-0 font-semibold text-contrast">
							{{ contentPending ? 'Loading…' : contentFailed ? 'Failed to load' : contentSummary }}
						</dd>
					</div>
					<div
						class="flex min-w-0 flex-col gap-0.5 border-0 border-t border-solid border-surface-4 px-4 py-3 lg:border-l lg:border-t-0"
					>
						<dt class="text-sm font-semibold text-secondary">Members</dt>
						<dd class="m-0 font-semibold text-contrast">{{ memberSummary }}</dd>
					</div>
					<div
						class="flex min-w-0 flex-col gap-0.5 border-0 border-l border-t border-solid border-surface-4 px-4 py-3 lg:border-t-0"
					>
						<dt class="text-sm font-semibold text-secondary">Owner</dt>
						<dd class="m-0 font-semibold text-contrast">{{ ownerSummaryText }}</dd>
					</div>
				</dl>
				<div
					class="flex flex-wrap items-center gap-2 border-0 border-t border-solid border-surface-4 bg-surface-2 px-4 py-3"
				>
					<Button v-if="contentFailed" :loading="contentFetching" @click="refetchContent()">
						Retry loading content
					</Button>
					<Button @click="openInspector">
						<EyeIcon />
						Inspect instance
					</Button>
					<Button type="colored" color="orange" @click="quarantineModal?.show()">
						<LockIcon />
						Quarantine
					</Button>
					<TeleportOverflowMenu
						label="More options"
						tooltip="More options"
						placement="bottom-start"
						:options="moreOptions"
					>
						<MoreHorizontalIcon />
					</TeleportOverflowMenu>
				</div>
			</div>
		</div>

		<SharedInstanceInspectorModal ref="inspectorModal" />
		<SharedInstanceQuarantineModal ref="quarantineModal" :instance="instanceSummary" />
		<SharedInstanceBanModal ref="banModal" :username="details.owner.username" subject="owner" />
	</section>
</template>
