<template>
	<PageHeader
		:header="project.title"
		:summary="project.description"
		:leading="leadingItem"
		:badges="headerBadges"
		:metadata="headerMetadata"
		:actions="actions"
		:disable-line-clamp="disableLineClamp"
	>
		<template #metadata-project-stats>
			<div class="flex items-center gap-3 flex-wrap gap-y-0">
				<template v-if="isServerProject">
					<ServerDetails
						v-if="projectV3?.status !== 'draft'"
						:online-players="playersOnline"
						:status-online="statusOnline"
						:recent-plays="javaServer?.verified_plays_2w ?? 0"
					/>
				</template>
				<template v-else>
					<div
						v-tooltip="
							capitalizeString(
								formatMessage(commonMessages.projectDownloads, {
									count: project.downloads,
								}),
							)
						"
						class="flex items-center gap-2 font-semibold cursor-help"
					>
						<DownloadIcon class="h-6 w-6 text-secondary" />
						{{ formatCompactNumber(project.downloads) }}
					</div>
					<div
						v-tooltip="
							capitalizeString(
								formatMessage(commonMessages.projectFollowers, {
									count: project.followers,
								}),
							)
						"
						class="flex items-center gap-2 cursor-help"
						:class="{ 'md:border-r': project.categories.length > 0 }"
					>
						<HeartIcon class="h-6 w-6 text-secondary" />
						<span class="font-semibold">
							{{ formatCompactNumber(project.followers) }}
						</span>
					</div>
				</template>
				<div v-if="project.categories.length > 0" class="hidden items-center gap-2 md:flex">
					<div class="flex flex-wrap gap-2">
						<TagItem
							v-for="(category, index) in project.categories"
							:key="index"
							:action="() => router.push(`${searchUrl}?f=categories:${category}`)"
						>
							<FormattedTag :tag="category" />
						</TagItem>
					</div>
				</div>
			</div>
		</template>
	</PageHeader>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon, HeartIcon } from '@modrinth/assets'
import { capitalizeString, type Project } from '@modrinth/utils'
import type { Component } from 'vue'
import { computed } from 'vue'
import type { RouteLocationRaw } from 'vue-router'
import { useRouter } from 'vue-router'

import { useCompactNumber, useVIntl } from '../../composables'
import { commonMessages } from '../../utils'
import FormattedTag from '../base/FormattedTag.vue'
import type { JoinedButtonAction } from '../base/JoinedButtons.vue'
import PageHeader from '../base/PageHeader.vue'
import TagItem from '../base/TagItem.vue'
import type { Item as TeleportOverflowMenuItem } from '../base/TeleportOverflowMenu.vue'
import ProjectStatusBadge from './ProjectStatusBadge.vue'
import ServerDetails from './server/ServerDetails.vue'

const router = useRouter()
const { formatMessage } = useVIntl()
const { formatCompactNumber } = useCompactNumber()

type HeaderAction = {
	id: string
	label: string
	component?: Component
	componentProps?: Record<string, unknown>
	class?: string
	icon?: Component
	iconProps?: Record<string, unknown>
	iconClass?: string
	tooltip?: string
	ariaLabel?: string
	to?: string | RouteLocationRaw
	onClick?: (event?: MouseEvent) => void | Promise<void>
	disabled?: boolean
	labelHidden?: boolean
	circular?: boolean
	color?: 'standard' | 'brand' | 'red' | 'orange' | 'green' | 'blue' | 'purple' | 'medal-promo'
	size?: 'standard' | 'large' | 'small'
	type?: 'standard' | 'outlined' | 'transparent' | 'highlight' | 'highlight-colored-text' | 'chip'
	joinedActions?: JoinedButtonAction[]
	menuActions?: TeleportOverflowMenuItem[]
	primaryDisabled?: boolean
	dropdownDisabled?: boolean
	primaryMuted?: boolean
}

const props = withDefaults(
	defineProps<{
		project: Project
		member?: boolean
		projectV3?: Labrinth.Projects.v3.Project | null
		ping?: number
		actions?: HeaderAction[]
		disableLineClamp?: boolean
	}>(),
	{
		member: false,
		actions: () => [],
		disableLineClamp: false,
	},
)

const leadingItem = computed(() => ({
	type: 'avatar' as const,
	src: props.project.icon_url,
	alt: props.project.title,
	avatarSize: '96px',
}))

const headerBadges = computed(() =>
	props.member || props.project.status !== 'approved'
		? [
				{
					id: 'status',
					component: ProjectStatusBadge,
					componentProps: {
						status: props.project.status,
					},
				},
			]
		: [],
)

const headerMetadata = [
	{
		id: 'project-stats',
		type: 'custom' as const,
		class: 'contents',
	},
]

const searchUrl = computed(
	() => `/discover/${isServerProject.value ? 'servers' : `${props.project.project_type}s`}`,
)

const isServerProject = computed(() => !!props.projectV3?.minecraft_server)
const javaServer = computed(() => props.projectV3?.minecraft_java_server)
const javaServerPingData = computed(() => props.projectV3?.minecraft_java_server?.ping?.data)
const playersOnline = computed(() => javaServerPingData.value?.players_online ?? 0)
const statusOnline = computed(() => !!javaServerPingData.value)
</script>
