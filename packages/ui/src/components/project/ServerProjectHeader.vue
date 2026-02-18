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
					v-if="serverFlag"
					:src="serverFlag"
					alt="Server region"
					class="h-5 w-8 rounded-sm object-cover border-surface-5 border border-solid"
				/>
				<div class="flex items-center gap-2 font-semibold">
					<UsersIcon class="h-5 w-5 text-secondary" />
					{{ serverPlayersOnline }}/{{ serverMaxPlayers }}
				</div>
				<TagItem
					class="border !border-solid border-brand bg-brand-highlight !font-medium"
					style="--_color: var(--color-brand)"
				>
					{{ ping }}ms
				</TagItem>
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
				<div
					v-if="serverLinkedModpack && modpackIconUrl"
					class="flex gap-1.5 items-center font-medium w-max"
				>
					<Avatar :src="modpackIconUrl" :alt="serverLinkedModpack" size="24px" />
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
import { HeartIcon, UsersIcon } from '@modrinth/assets'
import { formatNumber, type Project } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'
import { useRouter } from 'vue-router'

import type { Labrinth } from '@modrinth/api-client'
import { injectModrinthClient } from '../../providers/api-client'
import Avatar from '../base/Avatar.vue'
import ContentPageHeader from '../base/ContentPageHeader.vue'
import FormattedTag from '../base/FormattedTag.vue'
import TagItem from '../base/TagItem.vue'
import ProjectStatusBadge from './ProjectStatusBadge.vue'

const router = useRouter()
const { labrinth } = injectModrinthClient()

const props = withDefaults(
	defineProps<{
		project: Project
		projectV3: Labrinth.Projects.v3.Project | null
		member?: boolean
	}>(),
	{
		member: false,
	},
)

console.log(props.projectV3)

const minecraftServer = computed(() => props.projectV3?.minecraft_server)
const minecraftJavaServer = computed(() => props.projectV3?.minecraft_java_server)
const minecraftJavaServerPingData = computed(
	() => props.projectV3?.minecraft_java_server_ping?.data,
)
const serverFlag = computed(() =>
	minecraftServer.value?.country
		? `https://flagcdn.com/${minecraftServer.value.country.toLowerCase()}.svg`
		: '',
)
const serverPlayersOnline = computed(() => minecraftJavaServerPingData.value?.players_online ?? 0)
const serverMaxPlayers = computed(() => minecraftJavaServerPingData.value?.players_max ?? 0)
const serverModpackVersionId = computed(() =>
	minecraftJavaServer.value?.content?.kind === 'modpack'
		? minecraftJavaServer.value.content.version_id
		: null,
)

const ping = computed(() =>
	Math.trunc(Number(minecraftJavaServerPingData.value?.latency.nanos) / 1000000),
)

const { data: modpackVersion } = useQuery({
	queryKey: computed(() => ['modpack-version', serverModpackVersionId.value] as const),
	queryFn: () => labrinth.versions_v3.getVersion(serverModpackVersionId.value!),
	enabled: computed(() => !!serverModpackVersionId.value),
})

const modpackProjectId = computed(() => modpackVersion.value?.project_id ?? null)

const { data: modpackProject } = useQuery({
	queryKey: computed(() => ['modpack-project', modpackProjectId.value] as const),
	queryFn: () => labrinth.projects_v3.get(modpackProjectId.value!),
	enabled: computed(() => !!modpackProjectId.value),
})

const serverLinkedModpack = computed(() => modpackProject.value?.name ?? null)
const modpackIconUrl = computed(() => modpackProject.value?.icon_url ?? null)
</script>
