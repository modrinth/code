<template>
	<SmartClickable class="w-full project-card-container">
		<template v-if="link" #clickable>
			<AutoLink
				:to="link"
				class="rounded-xl no-outline no-click-animation custom-focus-indicator"
				@mouseenter="$emit('mouseenter')"
				@mouseleave="$emit('mouseleave')"
			></AutoLink>
		</template>
		<div v-if="layout === 'grid'" :class="[baseCardStyle, 'flex flex-col']">
			<div
				:style="{ '--_project-color': cssColor }"
				class="relative bg-project-gradient overflow-clip aspect-[2/1] w-full border-0 border-b-[1px] border-solid border-surface-4"
			>
				<img
					v-if="banner"
					:src="banner"
					alt=""
					class="absolute w-full h-full inset-0 object-cover object-center"
				/>
				<img
					v-else
					src="https://cdn-raw.modrinth.com/landing-new/landing.webp"
					alt=""
					class="absolute w-full h-full inset-0 object-cover object-center placeholder-banner scale-[200%]"
				/>
			</div>
			<div class="p-4 flex flex-col gap-3 grow">
				<div class="flex gap-3">
					<Avatar :src="iconUrl" size="96px" class="project-card__icon ease-brightness" no-shadow />
					<div class="flex flex-col gap-2 w-full">
						<div class="grid grid-cols-[1fr_auto] gap-4">
							<div class="flex flex-col gap-1">
								<div class="flex gap-2 items-center">
									<ProjectCardTitle :title="title" compact />
									<ProjectCardAuthor v-if="author" :author="author" />
									<ProjectStatusBadge v-if="status" :status="status" class="text-sm" />
								</div>
								<div class="m-0 font-normal line-clamp-2">
									{{ summary }}
								</div>
							</div>
						</div>
					</div>
				</div>
				<div class="flex gap-2 shrink-0 empty:hidden smart-clickable:allow-pointer-events">
					<slot name="actions" />
				</div>
				<div class="mt-auto flex flex-col gap-3 flex-wrap overflow-hidden justify-between grow">
					<div class="flex items-center gap-1 flex-wrap overflow-hidden">
						<ServerDetails
							v-if="isServerProject"
							:region="serverRegionCode"
							:online-players="serverOnlinePlayers"
							:recent-plays="serverRecentPlays"
							:ping="serverPing"
							:status-online="serverStatusOnline"
							:hide-online-players-label="true"
							:hide-recent-plays-label="true"
						/>
						<ProjectCardEnvironment
							v-if="environment"
							:client-side="environment.clientSide"
							:server-side="environment.serverSide"
						/>
						<ProjectCardTags
							v-if="tags"
							:tags="tags"
							:exclude-loaders="excludeLoaders"
							:deprioritized-tags="deprioritizedTags"
							:max-tags="(maxTags || 6) + (!!environment ? 0 : 1)"
						/>
						<ServerModpackContent
							v-if="serverModpackContent"
							:name="serverModpackContent.name"
							:icon="serverModpackContent.icon"
							:onclick="serverModpackContent.onclick"
							:show-custom-modpack-tooltip="serverModpackContent.showCustomModpackTooltip"
							class="text-primary"
						/>
					</div>
					<div
						v-if="downloads !== undefined || followers !== undefined"
						class="flex items-center gap-3 justify-between flex-wrap"
					>
						<div class="flex items-center gap-3 no-wrap flex-wrap">
							<ProjectCardStats :downloads="downloads" :followers="followers" />
						</div>
						<ProjectCardDate v-if="date && autoDisplayDate" :type="autoDisplayDate" :date="date" />
					</div>
				</div>
			</div>
		</div>
		<div
			v-else
			:class="[
				baseCardStyle,
				'p-4 grid grid-project-card-list gap-x-3 gap-y-2',
				{ 'has-actions': !!$slots.actions },
			]"
		>
			<Avatar
				:src="iconUrl"
				size="100px"
				class="project-card__icon grid-project-card-list__icon ease-brightness"
				no-shadow
			/>
			<div class="flex flex-col gap-2 grid-project-card-list__info">
				<div class="flex gap-2 items-center">
					<ProjectCardTitle :title="title" />
					<ProjectCardAuthor v-if="author" :author="author" />
					<ProjectStatusBadge v-if="status" :status="status" />
				</div>
				<div class="project-card-summary m-0 font-normal line-clamp-2">
					{{ summary }}
				</div>
			</div>

			<div
				v-if="!!$slots.actions"
				class="flex gap-1 shrink-0 ml-auto empty:hidden smart-clickable:allow-pointer-events grid-project-card-list__actions"
			>
				<slot name="actions" />
			</div>
			<div
				class="flex flex-col gap-3 items-end shrink-0 ml-auto empty:hidden grid-project-card-list__stats"
				:class="{ 'mt-3': !!$slots.actions }"
			>
				<div class="flex items-center gap-3">
					<ProjectCardStats :downloads="downloads" :followers="followers" />
				</div>
				<ProjectCardDate v-if="date && autoDisplayDate" :type="autoDisplayDate" :date="date" />
			</div>
			<div class="mt-auto flex items-center gap-3 grid-project-card-list__tags">
				<div class="flex items-center gap-2 flex-wrap">
					<ServerDetails
						v-if="isServerProject"
						:region="serverRegionCode"
						:online-players="serverOnlinePlayers"
						:status-online="serverStatusOnline"
						:recent-plays="serverRecentPlays"
						:ping="serverPing"
						:hide-online-players-label="true"
						:hide-recent-plays-label="true"
					/>
					<div class="flex items-center gap-1 flex-wrap">
						<ProjectCardEnvironment
							v-if="environment"
							:client-side="environment.clientSide"
							:server-side="environment.serverSide"
						/>
						<ProjectCardTags
							v-if="tags"
							:tags="tags"
							:extra-tags="extraTags"
							:exclude-loaders="excludeLoaders"
							:deprioritized-tags="deprioritizedTags"
							:max-tags="(maxTags || (!!$slots.actions ? 4 : 5)) + (!!environment ? 0 : 1)"
						/>
					</div>
					<ServerModpackContent
						v-if="serverModpackContent"
						:name="serverModpackContent.name"
						:icon="serverModpackContent.icon"
						:onclick="serverModpackContent.onclick"
						:show-custom-modpack-tooltip="serverModpackContent.showCustomModpackTooltip"
						class="text-primary"
					/>
				</div>
			</div>
		</div>
	</SmartClickable>
</template>

<script setup lang="ts">
import type { ProjectStatus } from '@modrinth/utils'
import dayjs from 'dayjs'
import { computed } from 'vue'

import { AutoLink, Avatar } from '../../base'
import { SmartClickable } from '../../base/index.ts'
import ProjectStatusBadge from '../ProjectStatusBadge.vue'
import ServerDetails from '../server/ServerDetails.vue'
import ServerModpackContent from '../server/ServerModpackContent.vue'
import ProjectCardAuthor from './ProjectCardAuthor.vue'
import ProjectCardDate from './ProjectCardDate.vue'
import ProjectCardEnvironment, {
	type ProjectCardEnvironmentProps,
} from './ProjectCardEnvironment.vue'
import ProjectCardStats from './ProjectCardStats.vue'
import ProjectCardTags from './ProjectCardTags.vue'
import ProjectCardTitle from './ProjectCardTitle.vue'

defineEmits<{
	mouseenter: []
	mouseleave: []
}>()

const props = defineProps<{
	layout: 'list' | 'grid'
	link?: string | (() => void)
	iconUrl?: string
	title: string
	author?: {
		name: string
		link?: string
	}
	summary?: string
	tags?: string[]
	allTags?: string[]
	deprioritizedTags?: string[]
	excludeLoaders?: boolean
	downloads?: number
	followers?: number
	dateUpdated?: string
	datePublished?: string
	displayedDate?: 'updated' | 'published'
	serverRegionCode?: string
	serverOnlinePlayers?: number
	serverStatusOnline?: boolean
	serverRecentPlays?: number
	serverPing?: number
	serverModpackContent?: {
		name: string
		icon?: string
		onclick?: () => void
		showCustomModpackTooltip?: boolean
	}
	isServerProject?: boolean
	banner?: string
	color?: string | number
	environment?: ProjectCardEnvironmentProps
	status?: ProjectStatus
	maxTags?: number
}>()

const baseCardStyle =
	'w-full h-full border-[1px] border-solid border-surface-4 overflow-hidden bg-surface-3 rounded-2xl transition-all smart-clickable:outline-on-focus smart-clickable:highlight-on-hover'

const updatedDate = computed(() =>
	props.dateUpdated ? dayjs(props.dateUpdated).toDate() : undefined,
)
const publishedDate = computed(() =>
	props.datePublished ? dayjs(props.datePublished).toDate() : undefined,
)

const autoDisplayDate = computed(() => {
	if (props.displayedDate) {
		return props.displayedDate
	} else if (props.dateUpdated) {
		return 'updated'
	} else if (props.datePublished) {
		return 'published'
	} else {
		return undefined
	}
})

const date = computed(() => {
	if (autoDisplayDate.value === 'updated') {
		return updatedDate.value
	} else if (autoDisplayDate.value === 'published') {
		return publishedDate.value
	}
	return undefined
})

const extraTags = computed(() => props.allTags?.filter((tag) => !props.tags?.includes(tag)))

const cssColor = computed(() => {
	if (props.color === undefined || typeof props.color === 'string') {
		return props.color
	}

	const color = props.color >>> 0
	const b = color & 0xff
	const g = (color & 0xff00) >>> 8
	const r = (color & 0xff0000) >>> 16
	return 'rgba(' + [r, g, b, 1].join(',') + ')'
})
</script>
<style scoped>
.no-outline {
	outline: none;
}

:deep(.project-card-container) {
	container-type: inline-size;
}

.grid-project-card-list {
	grid-template:
		'icon info stats stats'
		'icon info stats stats'
		'icon tags tags tags';
	grid-template-columns: auto 1fr auto auto;
}

.grid-project-card-list.has-actions {
	grid-template:
		'icon info actions actions'
		'icon info dummy stats'
		'icon tags tags stats';
	grid-template-columns: auto 1fr auto auto;
}

.grid-project-card-list__icon {
	grid-area: icon;
}

.grid-project-card-list__info {
	grid-area: info;
}

.grid-project-card-list__actions {
	grid-area: actions;
}

.grid-project-card-list__stats {
	grid-area: stats;
}

.grid-project-card-list__tags {
	grid-area: tags;
}

@container (width < 850px) {
	.project-card__icon {
		--_override-size: 64px;
	}

	.grid-project-card-list {
		grid-template:
			'icon info stats'
			'icon info stats'
			'tags tags tags';
		grid-template-columns: auto 1fr auto;
	}

	.grid-project-card-list.has-actions {
		grid-template:
			'icon info actions'
			'icon info stats'
			'tags tags stats';
		grid-template-columns: auto 1fr auto;
	}
}

@container (width < 550px) {
	.project-card__icon {
		--_override-size: 64px;
	}

	.grid-project-card-list {
		grid-template:
			'icon info'
			'icon info'
			'tags tags'
			'stats stats';
		grid-template-columns: auto 1fr;
	}

	.grid-project-card-list.has-actions {
		grid-template:
			'icon info'
			'icon info'
			'tags tags'
			'stats stats'
			'actions actions';
		grid-template-columns: auto 1fr;
	}

	.grid-project-card-list__stats,
	.grid-project-card-list__actions {
		@apply items-start w-full;
	}

	.grid-project-card-list__info {
		@apply gap-0.5;
	}

	.project-card-summary {
		@apply text-sm;
	}
}

/*noinspection CssUnresolvedCustomProperty*/
.bg-project-gradient {
	--_gradient-start: var(--_project-color, #000);
	--_gradient-end: var(--_project-color, #000);
	@supports (background-color: oklch(from var(--_project-color, #000) l c h)) {
		--_gradient-start: oklch(
			from var(--_project-color, #000) calc(l * 0.8) calc(c * 0.8) calc(h + 15)
		);
		--_gradient-end: oklch(from var(--_project-color, #000) calc(l * 0.5) calc(c * 0.9) h);
	}
	background-color: var(--_gradient-start);
	background-image: linear-gradient(to bottom right, var(--_gradient-start), var(--_gradient-end));
}

.placeholder-banner {
	opacity: 0.7;
}
</style>
