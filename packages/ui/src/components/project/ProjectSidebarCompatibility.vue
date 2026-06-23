<template>
	<div v-if="project.versions?.length > 0" class="flex flex-col gap-3">
		<h2 class="text-lg m-0">{{ formatMessage(messages.title) }}</h2>
		<section class="flex flex-col gap-2">
			<h3 class="text-primary !font-normal text-base m-0">
				{{ formatMessage(messages.minecraftJava) }}
			</h3>
			<div class="flex flex-wrap gap-1">
				<TagItem
					v-for="version in getVersionsToDisplay(project, tags.gameVersions)"
					:key="`version-tag-${version}`"
				>
					{{ version }}
				</TagItem>
			</div>
		</section>
		<section v-if="project.project_type !== 'resourcepack'" class="flex flex-col gap-2">
			<h3 class="text-primary !font-normal text-base m-0">
				{{ formatMessage(messages.platforms) }}
			</h3>
			<div class="flex flex-wrap gap-1">
				<template v-if="noModpackLoader">
					<TagItem class="border !border-solid border-surface-5 hover:no-underline">
						No mod loader
					</TagItem>
				</template>
				<template v-else>
					<TagItem
						v-for="platform in project.loaders"
						:key="`platform-tag-${platform}`"
						:action="() => router.push(`/${project.project_type}s?g=categories:${platform}`)"
						:style="`--_color: var(--color-platform-${platform})`"
					>
						<component :is="getLoaderIcon(platform)" v-if="getLoaderIcon(platform)" />
						<FormattedTag :tag="platform" enforce-type="loader" />
					</TagItem>
				</template>
			</div>
		</section>
		<section v-if="showEnvironments" class="flex flex-col gap-2">
			<h3 class="text-primary !font-normal text-base m-0">
				{{ formatMessage(messages.environments) }}
			</h3>
			<div class="flex flex-wrap gap-1">
				<EnvironmentTags :environment="primaryEnvironment" />
			</div>
		</section>
		<section
			v-else-if="
				(project.project_type === 'mod' || project.project_type === 'modpack') &&
				!(project.client_side === 'unsupported' && project.server_side === 'unsupported') &&
				!(project.client_side === 'unknown' && project.server_side === 'unknown')
			"
			class="flex flex-col gap-2"
		>
			<h3 class="text-primary text-base m-0">{{ formatMessage(messages.environments) }}</h3>
			<div class="flex flex-wrap gap-1">
				<TagItem
					v-if="
						(project.client_side === 'required' && project.server_side !== 'required') ||
						(project.client_side === 'optional' && project.server_side === 'optional')
					"
				>
					<ClientIcon aria-hidden="true" />
					Client-side
				</TagItem>
				<TagItem
					v-if="
						(project.server_side === 'required' && project.client_side !== 'required') ||
						(project.client_side === 'optional' && project.server_side === 'optional')
					"
				>
					<ServerIcon aria-hidden="true" />
					Server-side
				</TagItem>
				<TagItem v-if="false">
					<UserIcon aria-hidden="true" />
					Singleplayer
				</TagItem>
				<TagItem
					v-if="
						// @ts-ignore
						project.project_type !== 'datapack' &&
						project.client_side !== 'unsupported' &&
						project.server_side !== 'unsupported' &&
						project.client_side !== 'unknown' &&
						project.server_side !== 'unknown'
					"
				>
					<MonitorSmartphoneIcon aria-hidden="true" />
					Client and server
				</TagItem>
			</div>
		</section>
	</div>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	ClientIcon,
	getLoaderIcon,
	MonitorSmartphoneIcon,
	ServerIcon,
	UserIcon,
} from '@modrinth/assets'
import { FormattedTag, projectCompatibilityMessages, TagItem } from '@modrinth/ui'
import type { GameVersionTag, PlatformTag } from '@modrinth/utils'
import { getVersionsToDisplay } from '@modrinth/utils'
import { computed } from 'vue'
import { useRouter } from 'vue-router'

import { useVIntl } from '../../composables/i18n'
import EnvironmentTags from './EnvironmentTags.vue'

const { formatMessage } = useVIntl()
// TODO: anything in this component that uses the router will not work in the app. and this component is used in the app.
// fix is to replace any router stuff with click handlers and pass in the handlers as props from the parent component
const router = useRouter()

type EnvironmentValue = 'optional' | 'required' | 'unsupported' | 'unknown'

const TYPES_WITH_ENVS = ['mod', 'modpack'] as const

const props = defineProps<{
	project: {
		actualProjectType: string
		project_type: string
		loaders: string[]
		client_side: EnvironmentValue
		server_side: EnvironmentValue
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		versions: any[]
	}
	tags: {
		gameVersions: GameVersionTag[]
		loaders: PlatformTag[]
	}
	projectV3?: Labrinth.Projects.v3.Project
}>()

const noModpackLoader = computed(
	() =>
		(props.projectV3?.project_types.includes('modpack') &&
			props.projectV3?.mrpack_loaders?.length === 1 &&
			props.projectV3?.mrpack_loaders?.[0] === 'minecraft') ||
		props.projectV3?.mrpack_loaders?.length === 0,
)

const showEnvironments = computed(
	() =>
		TYPES_WITH_ENVS.some((x) => props.projectV3?.project_types.includes(x)) &&
		primaryEnvironment.value,
)

const primaryEnvironment = computed<Labrinth.Projects.v3.Environment>(
	() => props.projectV3?.environment?.find((x) => x !== 'unknown') ?? 'unknown',
)

const messages = projectCompatibilityMessages
</script>
