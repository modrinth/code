<template>
	<div class="shadow-card rounded-2xl border border-solid border-surface-4 bg-surface-3 p-4">
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-4">
				<NuxtLink :to="`/project/${projectRouteParam}`" target="_blank" tabindex="-1" class="flex">
					<Avatar
						:src="queueEntry.project.icon_url"
						size="4rem"
						class="rounded-2xl border border-surface-5 bg-surface-4 !shadow-none"
					/>
				</NuxtLink>
				<div class="flex flex-col gap-1.5">
					<div class="flex items-center gap-2">
						<NuxtLink
							:to="`/project/${projectRouteParam}`"
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
							class="flex items-center gap-2 rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1"
						>
							<span class="text-sm text-secondary">Requesting</span>
							<Badge
								:type="
									queueEntry.project.visibility === 'public'
										? 'approved'
										: queueEntry.project.visibility
								"
								class="text-sm"
							/>
						</div>
						<div
							v-if="showExternalDependencies"
							v-tooltip="'External dependencies'"
							class="flex items-center gap-1 rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1"
						>
							<FileIcon aria-hidden="true" class="size-4 text-secondary" />
							<span class="text-sm font-medium text-secondary">
								{{ queueEntry.external_dependencies_count }}
							</span>
						</div>
					</div>
					<div v-if="queueEntry.ownership" ref="ownerRow" class="flex flex-wrap items-center gap-2">
						<NuxtLink
							:to="`/${queueEntry.ownership.kind}/${queueEntry.ownership.id}`"
							target="_blank"
							class="flex w-fit items-center gap-1 text-sm font-medium text-secondary hover:underline"
						>
							<Avatar
								:src="queueEntry.ownership.icon_url"
								size="1.5rem"
								:circle="queueEntry.ownership.kind === 'user'"
								class="border border-surface-5 bg-surface-4 !shadow-none"
							/>
							{{ queueEntry.ownership.name }}
						</NuxtLink>
						<CreatorStatusStats
							v-if="ownerStatsRequested"
							:id="queueEntry.ownership.id"
							:kind="queueEntry.ownership.kind"
							:active="ownerRowVisible"
							class="!mt-0"
						/>
					</div>
				</div>
			</div>

			<div class="flex items-center gap-3">
				<div class="flex flex-col items-end">
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
					<span
						v-if="initialQueuedDate"
						v-tooltip="`First submitted ${formatDateTimeFull(initialQueuedDate.toDate())}`"
						class="text-sm text-secondary"
					>
						First submitted
						{{ formatRelativeTime(initialQueuedDate.toISOString()) }}
					</span>
				</div>

				<div class="flex items-center gap-2">
					<CopyCode v-tooltip="'Copy project ID'" :text="queueEntry.project.id" />
					<CopyLinkButton
						v-tooltip="'Copy project link'"
						:url="`https://modrinth.com/project/${queueEntry.project.id}`"
					/>
					<ButtonLink
						v-tooltip="'Open moderation thread'"
						:href="`/project/${queueEntry.project.id}/moderation`"
						target="_blank"
						circular
						icon-only
					>
						<ExternalIcon />
					</ButtonLink>
					<IconButton
						v-tooltip="'Begin review'"
						type="colored"
						color="orange"
						:label="'Begin review'"
						@click="openProjectForReview"
					>
						<ScaleIcon />
					</IconButton>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ExternalIcon, FileIcon, ScaleIcon } from '@modrinth/assets'
import {
	Avatar,
	Badge,
	ButtonLink,
	CopyCode,
	CopyLinkButton,
	getProjectTypeIcon,
	IconButton,
	useFormatDateTime,
	useRelativeTime,
} from '@modrinth/ui'
import { formatProjectType } from '@modrinth/utils'
import { useIntersectionObserver } from '@vueuse/core'
import dayjs from 'dayjs'
import { computed, ref, shallowRef, useTemplateRef } from 'vue'

import CreatorStatusStats from '~/components/CreatorStatusStats.vue'
import type { ModerationProject } from '~/helpers/moderation'

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
	showExternalDependencies?: boolean
}>()

const emit = defineEmits<{
	startFromProject: [projectId: string]
}>()

const ownerRow = useTemplateRef<HTMLDivElement>('ownerRow')
const ownerStatsRequested = ref(false)
const visibleOwnerRow = shallowRef<Element | null>(null)
const ownerRowVisible = computed(() => !!ownerRow.value && ownerRow.value === visibleOwnerRow.value)
useIntersectionObserver(ownerRow, (entries) => {
	const entry = entries.at(-1)
	if (!entry || entry.target !== ownerRow.value) return
	visibleOwnerRow.value = entry.isIntersecting ? entry.target : null
	if (entry.isIntersecting) ownerStatsRequested.value = true
})

function getDaysQueued(date: Date): number {
	const now = new Date()
	const diff = now.getTime() - date.getTime()
	return Math.floor(diff / (1000 * 60 * 60 * 24))
}

const queuedDate = computed(() => {
	return dayjs(
		props.queueEntry.project.queued ||
			props.queueEntry.project.published ||
			props.queueEntry.project.updated,
	)
})

const initialQueuedDate = computed(() => {
	const initialQueued = props.queueEntry.project.initial_queued
	if (!initialQueued) return null

	const date = dayjs(initialQueued)
	return date.isBefore(queuedDate.value.subtract(1, 'minute')) ? date : null
})

const daysInQueue = computed(() => {
	return getDaysQueued(queuedDate.value.toDate())
})

const projectRouteParam = computed(
	() => props.queueEntry.project.slug || props.queueEntry.project.id,
)

const formattedDate = computed(() => {
	const date =
		props.queueEntry.project.queued ||
		props.queueEntry.project.published ||
		props.queueEntry.project.updated
	if (!date) return 'Unknown'

	try {
		return formatRelativeTime(dayjs(date).toISOString())
	} catch {
		return 'Unknown'
	}
})

function openProjectForReview() {
	emit('startFromProject', props.queueEntry.project.id)
}
</script>
