<template>
	<div
		class="universal-card flex min-h-[6rem] flex-col justify-between gap-3 rounded-lg p-4 sm:h-24 sm:flex-row sm:items-center sm:gap-0"
	>
		<div class="flex min-w-0 flex-1 items-center gap-3">
			<div class="flex-shrink-0 rounded-lg">
				<Avatar size="48px" :src="queueEntry.project.icon_url" />
			</div>
			<div class="flex min-w-0 flex-1 flex-col">
				<h3 class="truncate text-lg font-semibold">
					{{ queueEntry.project.name }}
				</h3>
				<nuxt-link
					v-if="queueEntry.owner"
					target="_blank"
					class="flex items-center gap-1 truncate align-middle text-sm hover:text-brand"
					:to="`/user/${queueEntry.owner.user.username}`"
				>
					<Avatar
						:src="queueEntry.owner.user.avatar_url"
						circle
						size="16px"
						class="inline-block flex-shrink-0"
					/>
					<span class="truncate">{{ queueEntry.owner.user.username }}</span>
				</nuxt-link>
				<nuxt-link
					v-else-if="queueEntry.org"
					target="_blank"
					class="flex items-center gap-1 truncate align-middle text-sm hover:text-brand"
					:to="`/organization/${queueEntry.org.slug}`"
				>
					<Avatar
						:src="queueEntry.org.icon_url"
						circle
						size="16px"
						class="inline-block flex-shrink-0"
					/>
					<span class="truncate">{{ queueEntry.org.name }}</span>
				</nuxt-link>
			</div>
		</div>

		<div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:gap-4">
			<div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:gap-1">
				<span class="flex items-center gap-1 whitespace-nowrap text-sm">
					<BoxIcon
						v-if="queueEntry.project.project_type === 'mod'"
						class="size-4 flex-shrink-0"
						aria-hidden="true"
					/>
					<PaintbrushIcon
						v-else-if="queueEntry.project.project_type === 'resourcepack'"
						class="size-4 flex-shrink-0"
						aria-hidden="true"
					/>
					<BracesIcon
						v-else-if="queueEntry.project.project_type === 'datapack'"
						class="size-4 flex-shrink-0"
						aria-hidden="true"
					/>
					<PackageOpenIcon
						v-else-if="queueEntry.project.project_type === 'modpack'"
						class="size-4 flex-shrink-0"
						aria-hidden="true"
					/>
					<GlassesIcon
						v-else-if="queueEntry.project.project_type === 'shader'"
						class="size-4 flex-shrink-0"
						aria-hidden="true"
					/>
					<PlugIcon
						v-else-if="queueEntry.project.project_type === 'plugin'"
						class="size-4 flex-shrink-0"
						aria-hidden="true"
					/>
					<span class="hidden sm:inline">{{
						props.queueEntry.project.project_types.map(formatProjectType).join(', ')
					}}</span>
					<span class="sm:hidden">{{
						props.queueEntry.project.project_types.map(formatProjectType).join(', ')
					}}</span>
				</span>

				<span class="hidden text-sm sm:inline">&#x2022;</span>

				<div class="flex flex-row gap-2 text-sm">
					Requesting
					<Badge
						v-if="props.queueEntry.project.requested_status"
						:type="props.queueEntry.project.requested_status"
						class="status"
					/>
				</div>

				<span class="hidden text-sm sm:inline">&#x2022;</span>

				<span
					v-tooltip="`Since ${queuedDate.toLocaleString()}`"
					class="truncate text-sm"
					:class="{
						'text-red': daysInQueue > 4,
						'text-orange': daysInQueue > 2,
					}"
				>
					<span class="hidden sm:inline">{{ getSubmittedTime(queueEntry) }}</span>
					<span class="sm:hidden">{{
						getSubmittedTime(queueEntry).replace('Submitted ', '')
					}}</span>
				</span>
			</div>

			<div class="flex items-center justify-end gap-2 sm:justify-start">
				<ButtonStyled circular>
					<NuxtLink target="_blank" :to="`/project/${queueEntry.project.slug}`">
						<EyeIcon class="size-4" />
					</NuxtLink>
				</ButtonStyled>
				<ButtonStyled circular color="orange" @click="openProjectForReview">
					<button>
						<ScaleIcon class="size-4" />
					</button>
				</ButtonStyled>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	BoxIcon,
	BracesIcon,
	EyeIcon,
	GlassesIcon,
	PackageOpenIcon,
	PaintbrushIcon,
	PlugIcon,
	ScaleIcon,
} from '@modrinth/assets'
import { Avatar, Badge, ButtonStyled, useRelativeTime } from '@modrinth/ui'
import { formatProjectType } from '@modrinth/utils'
import dayjs from 'dayjs'
import { computed } from 'vue'

import type { ModerationProject } from '~/helpers/moderation'
import { useModerationStore } from '~/store/moderation.ts'

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

function getSubmittedTime(): string {
	const date =
		props.queueEntry.project.queued ||
		props.queueEntry.project.created ||
		props.queueEntry.project.updated
	if (!date) return 'Unknown'

	try {
		return `Submitted ${formatRelativeTime(dayjs(date).toISOString())}`
	} catch {
		return 'Unknown'
	}
}
</script>
