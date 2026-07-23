<script setup lang="ts">
import {
	BanIcon,
	BoxesIcon,
	BoxIcon,
	EyeIcon,
	HistoryIcon,
	LinkIcon,
	LockIcon,
	UserPlusIcon,
	UsersIcon,
	VersionIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	ConfirmLeaveModal,
	type ContentItem,
	ModpackContentModal,
	Table,
	type TableColumn,
	useFormatDateTime,
	useRelativeTime,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

export interface SharedInstanceReportUser {
	id: string
	username: string
	avatar_url?: string | null
	joined_at?: string | null
	last_played?: string | null
	join_type?: 'owner' | 'invite' | 'link'
}

export interface SharedInstanceReportVersion {
	version: number
	game_version?: string
	loader?: string
	loader_version?: string
	created?: string
}

export interface SharedInstanceOwnerInstance {
	id: string
	name: string
	icon_url?: string | null
	latest_version: number
	member_count: number
	quarantine: boolean
}

export interface SharedInstanceReportDetails {
	id: string
	name: string
	icon_url?: string | null
	quarantine: boolean
	owner: SharedInstanceReportUser
	members: SharedInstanceReportUser[]
	reported_version?: SharedInstanceReportVersion
	previous_versions: SharedInstanceReportVersion[]
	other_instances: SharedInstanceOwnerInstance[]
	other_instances_loaded?: boolean
}

const props = defineProps<{
	details: SharedInstanceReportDetails
	banPending?: boolean
	loadVersionContent?: (instanceId: string, version: number) => Promise<ContentItem[]>
}>()

const emit = defineEmits<{
	banOwner: [owner: SharedInstanceReportUser]
	contentError: [error: unknown]
}>()

const contentModal = ref<InstanceType<typeof ModpackContentModal> | null>(null)
const banModal = ref<InstanceType<typeof ConfirmLeaveModal> | null>(null)
const contentByVersion = new Map<string, ContentItem[]>()
const contentInstance = ref<Pick<SharedInstanceOwnerInstance, 'id' | 'name' | 'icon_url'>>({
	id: props.details.id,
	name: props.details.name,
	icon_url: props.details.icon_url,
})
const formatRelativeTime = useRelativeTime({ style: 'narrow' })
const formatDateTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })

type MemberTableColumn = 'username' | 'lastPlayed' | 'joined' | 'method'
type MemberTableRow = {
	id: string
	username: string
	avatarUrl?: string | null
	lastPlayed: string | null
	joined: string | null
	method: 'owner' | 'invite' | 'link'
}

const memberColumns: TableColumn<MemberTableColumn>[] = [
	{
		key: 'username',
		label: 'Username',
		width: 'clamp(14rem, 30%, 26rem)',
	},
	{
		key: 'lastPlayed',
		label: 'Last played',
		width: 'clamp(7rem, 15%, 13rem)',
		headerClass: 'whitespace-nowrap !px-2',
		cellClass: 'whitespace-nowrap !px-2',
	},
	{
		key: 'joined',
		label: 'Joined',
		width: 'clamp(7rem, 14%, 12rem)',
		headerClass: 'whitespace-nowrap !px-2',
		cellClass: 'whitespace-nowrap !px-2',
	},
	{
		key: 'method',
		label: 'Method',
		headerClass: 'whitespace-nowrap !px-2',
		cellClass: 'whitespace-nowrap !px-2',
	},
]

const allVersions = computed(() => [
	...(props.details.reported_version ? [props.details.reported_version] : []),
	...props.details.previous_versions,
])
const memberRows = computed<MemberTableRow[]>(() =>
	[props.details.owner, ...props.details.members].map((user) => ({
		id: user.id,
		username: user.username,
		avatarUrl: user.avatar_url,
		lastPlayed: user.last_played ?? null,
		joined: user.joined_at ?? null,
		method: user.join_type ?? 'invite',
	})),
)

async function promptBanOwner() {
	if (props.banPending || !(await banModal.value?.prompt())) return
	emit('banOwner', props.details.owner)
}

async function viewVersionContent(
	instance: Pick<SharedInstanceOwnerInstance, 'id' | 'name' | 'icon_url'>,
	version: number,
) {
	contentInstance.value = instance
	contentModal.value?.showLoading()

	try {
		const cacheKey = `${instance.id}:${version}`
		let content = contentByVersion.get(cacheKey)
		if (!content) {
			content = props.loadVersionContent ? await props.loadVersionContent(instance.id, version) : []
			contentByVersion.set(cacheKey, content)
		}
		contentModal.value?.setItems(content)
	} catch (error) {
		contentModal.value?.hide()
		emit('contentError', error)
	}
}

function viewReportedInstanceVersion(version: SharedInstanceReportVersion) {
	return viewVersionContent(
		{
			id: props.details.id,
			name: props.details.name,
			icon_url: props.details.icon_url,
		},
		version.version,
	)
}

function formattedLoader(version: SharedInstanceReportVersion) {
	const loader = version.loader
		? version.loader.charAt(0).toUpperCase() + version.loader.slice(1)
		: 'Vanilla'
	return version.loader_version ? `${loader} ${version.loader_version}` : loader
}
</script>

<template>
	<section
		class="shared-instance-message relative grid items-center gap-x-[var(--spacing-card-sm)] gap-y-[var(--spacing-card-xs)] break-words border-0 border-t border-solid border-surface-4 px-4 py-3"
	>
		<div
			class="shared-instance-message__icon backed-svg circle raised mb-auto border border-solid border-surface-5 shadow-[var(--shadow-card)]"
			style="--size: 2rem"
		>
			<BoxesIcon />
		</div>
		<div
			class="shared-instance-message__author flex shrink-0 flex-wrap items-center gap-[var(--spacing-card-xs)] font-bold text-[var(--color-text)]"
		>
			<span>Shared instance details</span>
			<span
				class="border-blue/60 rounded-full border border-solid bg-highlight-blue px-2 py-0.5 text-xs font-semibold text-blue"
			>
				Report context
			</span>
		</div>

		<div
			class="shared-instance-message__body overflow-hidden rounded-2xl border border-solid border-surface-4 bg-surface-3"
		>
			<div class="flex flex-col gap-3 p-4">
				<div class="flex items-center gap-2 font-semibold text-contrast">
					<UsersIcon class="size-5 text-secondary" />
					Owner and members
				</div>

				<Table :columns="memberColumns" :data="memberRows" row-key="id" table-min-width="42rem">
					<template #cell-username="{ row }">
						<NuxtLink
							:to="`/user/${row.username}`"
							target="_blank"
							class="inline-flex min-w-0 max-w-full items-center gap-2 text-primary hover:underline"
						>
							<Avatar
								:src="row.avatarUrl"
								:alt="`${row.username}'s avatar`"
								:tint-by="row.id"
								size="24px"
								circle
								no-shadow
							/>
							<span class="min-w-0 truncate font-medium">{{ row.username }}</span>
						</NuxtLink>
					</template>
					<template #cell-lastPlayed="{ row }">
						<span v-if="row.lastPlayed" v-tooltip="formatDateTime(row.lastPlayed)">
							{{ formatRelativeTime(row.lastPlayed) }}
						</span>
						<span v-else>Never</span>
					</template>
					<template #cell-joined="{ row }">
						<span v-if="row.joined" v-tooltip="formatDateTime(row.joined)">
							{{ formatRelativeTime(row.joined) }}
						</span>
						<span v-else>Unknown</span>
					</template>
					<template #cell-method="{ row }">
						<span class="inline-flex min-w-0 items-center gap-2">
							<UsersIcon v-if="row.method === 'owner'" class="size-5 shrink-0" aria-hidden="true" />
							<LinkIcon
								v-else-if="row.method === 'link'"
								class="size-5 shrink-0"
								aria-hidden="true"
							/>
							<UserPlusIcon v-else class="size-5 shrink-0" aria-hidden="true" />
							<span class="min-w-0 truncate">
								{{
									row.method === 'owner'
										? 'Owner'
										: row.method === 'link'
											? 'Share link'
											: 'Direct invite'
								}}
							</span>
						</span>
					</template>
				</Table>
			</div>

			<div class="border-0 border-t border-solid border-surface-4 p-4">
				<div class="flex flex-col gap-3">
					<div class="flex items-center gap-2 font-semibold text-contrast">
						<HistoryIcon class="size-5 text-secondary" />
						Version history
					</div>

					<div v-if="allVersions.length" class="grid max-h-64 gap-2 overflow-y-auto lg:grid-cols-2">
						<div
							v-for="version in allVersions"
							:key="version.version"
							class="flex flex-col gap-2 rounded-xl bg-surface-2 p-3 sm:flex-row sm:items-center sm:justify-between"
						>
							<div class="flex min-w-0 items-center gap-2.5">
								<div class="backed-svg shrink-0">
									<VersionIcon />
								</div>
								<div class="flex min-w-0 flex-col">
									<div class="flex flex-wrap items-center gap-2">
										<span class="font-semibold text-contrast">Version {{ version.version }}</span>
										<span
											v-if="version.version === details.reported_version?.version"
											class="bg-orange-highlight rounded-full px-2 py-0.5 text-xs font-semibold text-orange"
										>
											Reported
										</span>
									</div>
									<span v-if="version.game_version" class="truncate text-sm text-secondary">
										Minecraft {{ version.game_version }} · {{ formattedLoader(version) }}
									</span>
									<span v-else class="text-sm text-secondary">
										Content and metadata load on demand
									</span>
								</div>
							</div>

							<ButtonStyled type="outlined">
								<button class="w-full sm:w-auto" @click="viewReportedInstanceVersion(version)">
									<EyeIcon class="size-4" />
									View content
								</button>
							</ButtonStyled>
						</div>
					</div>
					<span v-else class="text-sm text-secondary">
						No version was attached to this report.
					</span>
				</div>
			</div>

			<div class="flex flex-col gap-3 border-0 border-t border-solid border-surface-4 p-4">
				<div class="flex items-center gap-2 font-semibold text-contrast">
					<BoxIcon class="size-5 text-secondary" />
					Other instances owned by {{ details.owner.username }}
				</div>

				<div v-if="details.other_instances.length" class="grid gap-2 sm:grid-cols-2 xl:grid-cols-3">
					<div
						v-for="instance in details.other_instances"
						:key="instance.id"
						class="flex min-w-0 items-center gap-2 rounded-xl bg-surface-2 p-3"
					>
						<Avatar :src="instance.icon_url" :tint-by="instance.id" size="2.5rem" no-shadow />
						<div class="flex min-w-0 flex-1 flex-col">
							<div class="flex min-w-0 items-center gap-1.5">
								<span class="truncate font-semibold text-contrast">{{ instance.name }}</span>
								<LockIcon
									v-if="instance.quarantine"
									v-tooltip="'Quarantined'"
									class="size-4 shrink-0 text-orange"
								/>
							</div>
							<span class="text-sm text-secondary">
								Version {{ instance.latest_version }} · {{ instance.member_count }}
								{{ instance.member_count === 1 ? 'member' : 'members' }}
							</span>
						</div>
						<ButtonStyled circular type="transparent">
							<button
								v-tooltip="`View ${instance.name} version ${instance.latest_version} content`"
								:aria-label="`View ${instance.name} version ${instance.latest_version} content`"
								@click="viewVersionContent(instance, instance.latest_version)"
							>
								<EyeIcon class="size-4" />
							</button>
						</ButtonStyled>
					</div>
				</div>
				<span v-else-if="details.other_instances_loaded" class="text-sm text-secondary">
					This owner has no other shared instances.
				</span>
				<span v-else class="text-sm text-secondary">
					Other instances are unavailable from the current shared instances API.
				</span>
			</div>

			<footer
				class="flex flex-col gap-3 border-0 border-t border-solid border-surface-4 bg-surface-2 p-4 sm:flex-row sm:items-center sm:justify-between"
			>
				<div class="flex max-w-2xl flex-col gap-0.5">
					<span class="font-semibold text-contrast">Shared instances service ban</span>
					<span class="text-sm text-secondary">
						Banning this owner quarantines all of their shared instances and prevents members from
						launching them.
					</span>
				</div>
				<ButtonStyled color="red">
					<button :disabled="banPending" class="w-full gap-2 sm:w-auto" @click="promptBanOwner">
						<BanIcon class="size-4" />
						{{ banPending ? 'Banning owner…' : 'Ban from shared instances' }}
					</button>
				</ButtonStyled>
			</footer>
		</div>

		<ModpackContentModal
			ref="contentModal"
			header="Shared instance content"
			:modpack-name="contentInstance.name"
			:modpack-icon-url="contentInstance.icon_url ?? undefined"
		/>
		<ConfirmLeaveModal
			ref="banModal"
			title="Ban from shared instances?"
			:header="`Ban ${details.owner.username} from shared instances`"
			body="This permanently prevents the owner from creating or managing shared instances. All of their shared instances will be quarantined, and members will be unable to launch them."
			stay-label="Cancel"
			leave-label="Ban owner"
			admonition-type="critical"
		/>
	</section>
</template>

<style scoped>
.shared-instance-message {
	grid-template:
		'icon author'
		'icon body';
	grid-template-columns: min-content minmax(0, 1fr);
	grid-template-rows: min-content 1fr;
}

.shared-instance-message__icon {
	grid-area: icon;
}

.shared-instance-message__author {
	grid-area: author;
}

.shared-instance-message__body {
	grid-area: body;
}
</style>
