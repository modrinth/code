<template>
	<div v-if="project.versions.length > 0" class="flex flex-col gap-3">
		<h2 class="text-lg m-0">{{ formatMessage(messages.title) }}</h2>
		<section class="flex flex-col gap-2">
			<h3 class="text-primary text-base m-0">{{ formatMessage(messages.minecraftJava) }}</h3>
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
			<h3 class="text-primary text-base m-0">{{ formatMessage(messages.platforms) }}</h3>
			<div class="flex flex-wrap gap-1">
				<TagItem
					v-for="platform in project.loaders"
					:key="`platform-tag-${platform}`"
					:action="() => router.push(`/${project.project_type}s?g=categories:${platform}`)"
					:style="`--_color: var(--color-platform-${platform})`"
				>
					<svg v-html="tags.loaders.find((x) => x.name === platform).icon"></svg>
					{{ formatCategory(platform) }}
				</TagItem>
			</div>
		</section>
		<section v-if="showEnvironments" class="flex flex-col gap-2">
			<h3 class="text-primary text-base m-0">{{ formatMessage(messages.environments) }}</h3>
			<div class="flex flex-wrap gap-1">
				<TagItem v-for="tag in primaryEnvironmentTags" :key="`environment-tag-${tag.message.id}`">
					<component :is="tag.icon" />
					{{ formatMessage(tag.message) }}
				</TagItem>
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
import { ClientIcon, MonitorSmartphoneIcon, ServerIcon, UserIcon } from '@modrinth/assets'
import type { EnvironmentV3, GameVersionTag, PlatformTag, ProjectV3Partial } from '@modrinth/utils'
import { formatCategory, getVersionsToDisplay } from '@modrinth/utils'
import { defineMessage, defineMessages, type MessageDescriptor, useVIntl } from '@vintl/vintl'
import { type Component, computed } from 'vue'
import { useRouter } from 'vue-router'

import TagItem from '../base/TagItem.vue'

const { formatMessage } = useVIntl()
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
	v3Metadata?: ProjectV3Partial
}>()

const showEnvironments = computed(
	() =>
		TYPES_WITH_ENVS.some((x) => props.v3Metadata?.project_types.includes(x)) &&
		primaryEnvironment.value,
)

const primaryEnvironment = computed<EnvironmentV3 | undefined>(() =>
	props.v3Metadata?.environment?.find((x) => x !== 'unknown'),
)

type EnvironmentTag = {
	icon: Component
	message: MessageDescriptor
	environments: EnvironmentV3[]
}

const environmentTags: EnvironmentTag[] = [
	{
		icon: ClientIcon,
		message: defineMessage({
			id: `project.about.compatibility.environments.client-side`,
			defaultMessage: 'Client-side',
		}),
		environments: [
			'client_only',
			'client_only_server_optional',
			'client_or_server',
			'client_or_server_prefers_both',
		],
	},
	{
		icon: ServerIcon,
		message: defineMessage({
			id: `project.about.compatibility.environments.server-side`,
			defaultMessage: 'Server-side',
		}),
		environments: [
			'server_only',
			'server_only_client_optional',
			'client_or_server',
			'client_or_server_prefers_both',
		],
	},
	{
		icon: ServerIcon,
		message: defineMessage({
			id: `project.about.compatibility.environments.dedicated-servers-only`,
			defaultMessage: 'Dedicated servers only',
		}),
		environments: ['dedicated_server_only'],
	},
	{
		icon: UserIcon,
		message: defineMessage({
			id: `project.about.compatibility.environments.singleplayer-only`,
			defaultMessage: 'Singleplayer only',
		}),
		environments: ['singleplayer_only'],
	},
	{
		icon: UserIcon,
		message: defineMessage({
			id: `project.about.compatibility.environments.singleplayer`,
			defaultMessage: 'Singleplayer',
		}),
		environments: ['server_only'],
	},
	{
		icon: MonitorSmartphoneIcon,
		message: defineMessage({
			id: `project.about.compatibility.environments.client-and-server`,
			defaultMessage: 'Client and server',
		}),
		environments: [
			'client_and_server',
			'client_only_server_optional',
			'server_only_client_optional',
			'client_or_server_prefers_both',
		],
	},
]

const primaryEnvironmentTags = computed(() => {
	return primaryEnvironment.value
		? environmentTags.filter((x) => x.environments.includes(primaryEnvironment.value ?? 'unknown'))
		: []
})

const messages = defineMessages({
	title: {
		id: `project.about.compatibility.title`,
		defaultMessage: 'Compatibility',
	},
	minecraftJava: {
		id: `project.about.compatibility.game.minecraftJava`,
		defaultMessage: 'Minecraft: Java Edition',
	},
	platforms: {
		id: `project.about.compatibility.platforms`,
		defaultMessage: 'Platforms',
	},
	environments: {
		id: `project.about.compatibility.environments`,
		defaultMessage: 'Supported environments',
	},
})
</script>
