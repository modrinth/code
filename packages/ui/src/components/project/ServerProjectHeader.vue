<template>
	<ContentPageHeader>
		<template #icon>
			<Avatar :src="project.icon_url" :alt="project.title" size="96px" />
		</template>
		<template #title>
			{{ project.title }}
		</template>
		<template #title-suffix>
			<ProjectStatusBadge v-if="member || project.status !== 'approved'" :status="project.status" />
		</template>
		<template #summary>
			{{ project.description }}
		</template>
		<template #stats>
			<div class="flex items-center gap-3 gap-y-1 flex-wrap">
				<img
					:src="serverFlag"
					alt="Server region"
					class="h-5 w-8 rounded-sm object-cover border-surface-5 border border-solid"
				/>
				<div class="flex items-center gap-2 font-semibold">
					<UsersIcon class="h-5 w-5 text-secondary" />
					{{ serverPlayersOnline }}/{{ serverMaxPlayers }}
				</div>
				<div class="flex items-center gap-2 font-semibold">
					<SignalIcon class="h-5 w-5 text-brand-green" />
				</div>
				<div
					v-tooltip="
						`${formatNumber(project.followers, false)} follower${project.followers !== 1 ? 's' : ''}`
					"
					class="flex items-center gap-2 cursor-help"
				>
					<HeartIcon class="h-5 w-5 text-secondary" />
					<span class="font-semibold">
						{{ formatNumber(project.followers) }}
					</span>
				</div>
				<div v-if="project.categories.length > 0" class="hidden items-center gap-2 md:flex">
					<div class="flex gap-2">
						<TagItem
							v-for="(category, index) in project.categories"
							:key="index"
							:action="() => router.push(`/${project.project_type}s?f=categories:${category}`)"
						>
							<FormattedTag :tag="category" />
						</TagItem>
					</div>
				</div>
				<div v-if="serverLinkedModpack" class="flex gap-1.5 items-center font-medium w-max">
					<Avatar :src="project.icon_url" :alt="serverLinkedModpack" size="24px" />
					{{ serverLinkedModpack }}
				</div>
			</div>
		</template>
		<template #actions>
			<slot name="actions" />
		</template>
	</ContentPageHeader>
</template>
<script setup lang="ts">
import { HeartIcon, SignalIcon, UsersIcon } from '@modrinth/assets'
import { formatNumber, type Project } from '@modrinth/utils'
import { computed } from 'vue'
import { useRouter } from 'vue-router'

import Avatar from '../base/Avatar.vue'
import ContentPageHeader from '../base/ContentPageHeader.vue'
import FormattedTag from '../base/FormattedTag.vue'
import TagItem from '../base/TagItem.vue'
import ProjectStatusBadge from './ProjectStatusBadge.vue'

const router = useRouter()

const props = withDefaults(
	defineProps<{
		project: Project
		member?: boolean
	}>(),
	{
		member: false,
	},
)

// TODO: replace mocks with real data from project
const serverFlag = computed(() => `https://flagcdn.com/us.svg`)
const serverPlayersOnline = computed(() => 326)
const serverMaxPlayers = computed(() => 3000)
const serverLinkedModpack = computed(() => 'Cobblemon Modpack')
</script>
