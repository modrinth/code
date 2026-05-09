<template>
	<div class="shadow-card rounded-2xl border border-solid border-surface-4 bg-surface-3 p-4">
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-4">
				<NuxtLink
					:to="`/project/${queueEntry.project.slug}`"
					target="_blank"
					tabindex="-1"
					class="flex"
				>
					<Avatar
						:src="queueEntry.project.icon_url"
						size="4rem"
						class="rounded-2xl border border-surface-5 bg-surface-4 !shadow-none"
					/>
				</NuxtLink>
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
								class="size-4"
							/>
							<span class="text-sm font-medium text-secondary">
								{{
									queueEntry.project.project_types.length === 0
										? '???'
										: queueEntry.project.project_types
												.map((t) => formatProjectType(t, true))
												.join(', ')
								}}
							</span>
						</div>
						<div
							v-if="queueEntry.project.requested_status"
							class="flex items-center gap-2 rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1"
						>
							<span class="text-sm text-secondary">Requesting</span>
							<Badge :type="queueEntry.project.requested_status" class="text-sm" />
						</div>
					</div>
					<div v-if="queueEntry.ownership?.kind === 'user'">
						<NuxtLink
							:to="`/user/${queueEntry.ownership.id}`"
							target="_blank"
							class="flex w-fit min-w-40 items-center gap-1 text-sm font-medium text-secondary hover:underline"
						>
							<Avatar
								:src="queueEntry.ownership.icon_url"
								size="1.5rem"
								circle
								class="border border-surface-5 bg-surface-4 !shadow-none"
							/>
							{{ queueEntry.ownership.name }}
						</NuxtLink>
					</div>
					<div
						v-else-if="queueEntry.ownership?.kind === 'organization'"
						class="flex items-center gap-1"
					>
						<Avatar
							:src="queueEntry.ownership.icon_url"
							size="1.5rem"
							class="border border-surface-5 bg-surface-4 !shadow-none"
						/>
						<NuxtLink
							:to="`/organization/${queueEntry.ownership.id}`"
							target="_blank"
							class="text-sm font-medium text-secondary hover:underline"
						>
							{{ queueEntry.ownership.name }}
						</NuxtLink>
					</div>
				</div>
			</div>

			<div class="flex items-center gap-3">
				<span
					v-tooltip="`Since ${formatDateTimeFull(queuedDate.toDate())}`"
					class="text-base text-secondary"
					:class="{
						'text-red': daysInQueue > 4,
						'text-orange': daysInQueue > 2 && daysInQueue <= 4,
					}"
				>
					{{ formattedDate }}
				</span>

				<div class="flex items-center gap-2">
					<ButtonStyled circular>
						<button v-tooltip="'Copy ID'" @click="copyId">
							<ClipboardCopyIcon />
						</button>
					</ButtonStyled>
					<ButtonStyled circular>
						<button v-tooltip="'Copy link'" @click="copyLink">
							<LinkIcon />
						</button>
					</ButtonStyled>
					<ButtonStyled circular color="orange">
						<button v-tooltip="'Begin review'" @click="openProjectForReview">
							<ScaleIcon />
						</button>
					</ButtonStyled>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ClipboardCopyIcon, LinkIcon, ScaleIcon } from '@modrinth/assets'
import {
	Avatar,
	Badge,
	ButtonStyled,
	getProjectTypeIcon,
	injectNotificationManager,
	useFormatDateTime,
	useRelativeTime,
} from '@modrinth/ui'
import { formatProjectType } from '@modrinth/utils'
import dayjs from 'dayjs'
import { computed } from 'vue'

import type { ModerationProject } from '~/helpers/moderation'

const { addNotification } = injectNotificationManager()
const formatRelativeTime = useRelativeTime()
const formatDateTimeFull = useFormatDateTime({
	weekday: 'short',
	year: 'numeric',
	month: 'short',
	day: 'numeric',
	hour: 'numeric',
	minute: '2-digit',
	second: '2-digit',
	timeZoneName: 'short',
	timeZone: 'UTC',
})

const props = defineProps<{
	queueEntry: ModerationProject
}>()

const emit = defineEmits<{
	startFromProject: [projectId: string]
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

function copyLink() {
	const base = window.location.origin
	const projectUrl = `${base}/project/${props.queueEntry.project.slug}`
	navigator.clipboard.writeText(projectUrl).then(() => {
		addNotification({
			type: 'success',
			title: 'Project link copied',
			text: 'The link to this project has been copied to your clipboard.',
		})
	})
}

function copyId() {
	navigator.clipboard.writeText(props.queueEntry.project.id).then(() => {
		addNotification({
			type: 'success',
			title: 'Project ID copied',
			text: 'The ID of this project has been copied to your clipboard.',
		})
	})
}

function openProjectForReview() {
	emit('startFromProject', props.queueEntry.project.id)
}
</script>
