<template>
	<div class="shadow-card rounded-2xl border border-surface-5 bg-surface-3 p-4">
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-4">
				<Avatar
					:src="queueEntry.project.icon_url"
					size="4rem"
					class="rounded-2xl border border-surface-5 bg-surface-4 !shadow-none"
				/>
				<div class="flex flex-col gap-1.5">
					<div class="flex items-center gap-2">
						<NuxtLink
							:to="`/project/${queueEntry.project.slug}`"
							target="_blank"
							class="text-lg font-semibold text-contrast hover:underline"
						>
							{{ queueEntry.project.name }}
						</NuxtLink>
						<div
							class="flex items-center gap-1 rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1"
						>
							<component
								:is="getProjectTypeIcon(queueEntry.project.project_types[0] as any)"
								aria-hidden="true"
								class="h-4 w-4"
							/>
							<span class="text-sm font-medium text-secondary">
								{{
									queueEntry.project.project_types.map((t) => formatProjectType(t, true)).join(', ')
								}}
							</span>
						</div>
						<div
							v-if="queueEntry.project.requested_status"
							class="flex items-center gap-2 rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1"
						>
							<span class="text-sm text-secondary">Requesting</span>
							<Badge :type="queueEntry.project.requested_status" class="status" />
						</div>
					</div>
					<div v-if="queueEntry.owner" class="flex items-center gap-1">
						<Avatar
							:src="queueEntry.owner.user.avatar_url"
							size="1.5rem"
							circle
							class="border border-surface-5 bg-surface-4 !shadow-none"
						/>
						<NuxtLink
							:to="`/user/${queueEntry.owner.user.username}`"
							target="_blank"
							class="text-sm font-medium text-secondary hover:underline"
						>
							{{ queueEntry.owner.user.username }}
						</NuxtLink>
					</div>
					<div v-else-if="queueEntry.org" class="flex items-center gap-1">
						<Avatar
							:src="queueEntry.org.icon_url"
							size="1.5rem"
							circle
							class="border border-surface-5 bg-surface-4 !shadow-none"
						/>
						<NuxtLink
							:to="`/organization/${queueEntry.org.slug}`"
							target="_blank"
							class="text-sm font-medium text-secondary hover:underline"
						>
							{{ queueEntry.org.name }}
						</NuxtLink>
					</div>
				</div>
			</div>

			<div class="flex items-center gap-3">
				<span
					v-tooltip="`Since ${queuedDate.toLocaleString()}`"
					class="text-base text-secondary"
					:class="{
						'text-red': daysInQueue > 4,
						'text-orange': daysInQueue > 2 && daysInQueue <= 4,
					}"
				>
					{{ formattedDate }}
				</span>

				<div class="flex items-center gap-2">
					<ButtonStyled circular color="orange">
						<button @click="openProjectForReview">
							<ScaleIcon class="size-5" />
						</button>
					</ButtonStyled>
					<ButtonStyled circular>
						<OverflowMenu :options="quickActions">
							<template #default>
								<EllipsisVerticalIcon class="size-4" />
							</template>
							<template #copy-id>
								<ClipboardCopyIcon />
								<span class="hidden sm:inline">Copy ID</span>
							</template>
							<template #copy-link>
								<LinkIcon />
								<span class="hidden sm:inline">Copy link</span>
							</template>
						</OverflowMenu>
					</ButtonStyled>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ClipboardCopyIcon, EllipsisVerticalIcon, LinkIcon, ScaleIcon } from '@modrinth/assets'
import {
	Avatar,
	Badge,
	ButtonStyled,
	getProjectTypeIcon,
	injectNotificationManager,
	OverflowMenu,
	type OverflowMenuOption,
	useRelativeTime,
} from '@modrinth/ui'
import { formatProjectType } from '@modrinth/utils'
import dayjs from 'dayjs'
import { computed } from 'vue'

import type { ModerationProject } from '~/helpers/moderation'
import { useModerationStore } from '~/store/moderation.ts'

const { addNotification } = injectNotificationManager()
const formatRelativeTime = useRelativeTime()
const moderationStore = useModerationStore()

const props = defineProps<{
	queueEntry: ModerationProject
}>()

function getDaysQueued(date: Date): number {
	const now = new Date()
	const diff = now.getTime() - date.getTime()
	return Math.floor(diff / (1000 * 60 * 60 * 24))
}

const queuedDate = computed(() => {
	return dayjs(
		props.queueEntry.project.queued ||
			props.queueEntry.project.created ||
			props.queueEntry.project.updated,
	)
})

const daysInQueue = computed(() => {
	return getDaysQueued(queuedDate.value.toDate())
})

const formattedDate = computed(() => {
	const date =
		props.queueEntry.project.queued ||
		props.queueEntry.project.created ||
		props.queueEntry.project.updated
	if (!date) return 'Unknown'

	try {
		return formatRelativeTime(dayjs(date).toISOString())
	} catch {
		return 'Unknown'
	}
})

const quickActions: OverflowMenuOption[] = [
	{
		id: 'copy-link',
		action: () => {
			const base = window.location.origin
			const projectUrl = `${base}/project/${props.queueEntry.project.slug}`
			navigator.clipboard.writeText(projectUrl).then(() => {
				addNotification({
					type: 'success',
					title: 'Project link copied',
					text: 'The link to this project has been copied to your clipboard.',
				})
			})
		},
	},
	{
		id: 'copy-id',
		action: () => {
			navigator.clipboard.writeText(props.queueEntry.project.id).then(() => {
				addNotification({
					type: 'success',
					title: 'Project ID copied',
					text: 'The ID of this project has been copied to your clipboard.',
				})
			})
		},
	},
]

function openProjectForReview() {
	moderationStore.setSingleProject(props.queueEntry.project.id)
	navigateTo({
		name: 'type-id',
		params: {
			type: 'project',
			id: props.queueEntry.project.id,
		},
		state: {
			showChecklist: true,
		},
	})
}
</script>
