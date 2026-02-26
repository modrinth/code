<template>
	<div v-if="hasContent" class="flex flex-col gap-3">
		<h2 class="text-lg m-0">{{ formatMessage(messages.title) }}</h2>

		<div
			v-if="ipAddress"
			v-tooltip="`Copy Java IP: ${ipAddress}`"
			class="bg-button-bg flex gap-2 justify-between rounded-2xl items-center px-3 pr-1.5 h-12 cursor-pointer hover:bg-button-bg-hover transition-colors"
			@click="handleCopyIP"
		>
			<div class="font-semibold truncate">
				{{ ipAddress }}
			</div>
			<div class="w-9 h-9 grid place-content-center">
				<CopyIcon class="shrink-0" />
			</div>
		</div>

		<section v-if="requiredContent" class="flex flex-col gap-2">
			<h3 class="text-primary text-base m-0">Required content</h3>
			<ServerModpackContentCard
				:name="requiredContent.name"
				:version-number="requiredContent.versionNumber ?? ''"
				:icon="requiredContent.icon"
				:onclick-name="requiredContent.onclickName"
				:onclick-version="requiredContent.onclickVersion"
				:onclick-download="requiredContent.onclickDownload"
			/>
		</section>
		<section v-if="recommendedVersions.length" class="flex flex-col gap-2">
			<h3 class="text-primary text-base m-0">Minecraft: Java Edition</h3>
			<div class="flex flex-wrap gap-1.5">
				<TagItem
					v-for="version in formatVersionsForDisplay(recommendedVersions, tags.gameVersions)"
					:key="`recommended-tag-${version}`"
				>
					{{ version }}
					<template v-if="supportedVersions.length > 0"> (Recommended) </template>
				</TagItem>
				<TagItem
					v-for="version in formatVersionsForDisplay(supportedVersionsList, tags.gameVersions)"
					:key="`supported-tag-${version}`"
				>
					{{ version }}
				</TagItem>
				<TagItem
					v-for="loader in loaders ?? []"
					:key="`loader-${loader}`"
					class="border !border-solid border-surface-5"
					:style="`--_color: var(--color-platform-${loader})`"
				>
					<component :is="getLoaderIcon(loader)" v-if="getLoaderIcon(loader)" />
					<FormattedTag :tag="loader" enforce-type="loader" />
				</TagItem>
			</div>
		</section>
		<section v-if="props.ping !== undefined || country" class="flex flex-col gap-2">
			<h3 class="text-primary text-base m-0">Country</h3>
			<div class="flex flex-wrap gap-1.5 items-center">
				<ServerRegion v-if="country" :region="country" />
				<ServerPing :ping="props.ping" />
			</div>
		</section>
		<section v-if="languages.length > 0" class="flex flex-col gap-2">
			<h3 class="text-primary text-base m-0">Languages</h3>
			<div class="flex flex-wrap gap-1.5">
				<TagItem v-for="language in languages" :key="`${language}`">
					{{ language }}
				</TagItem>
			</div>
		</section>
	</div>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CopyIcon, getLoaderIcon } from '@modrinth/assets'
import { formatVersionsForDisplay, type GameVersionTag, type PlatformTag } from '@modrinth/utils'
import { computed } from 'vue'

import { defineMessages, useVIntl } from '../../composables'
import { injectNotificationManager } from '../../providers'
import FormattedTag from '../base/FormattedTag.vue'
import TagItem from '../base/TagItem.vue'
import ServerModpackContentCard from './server/ServerModpackContentCard.vue'
import ServerPing from './server/ServerPing.vue'
import ServerRegion from './server/ServerRegion.vue'

interface RequiredContent {
	name: string
	versionNumber?: string
	icon?: string
	onclickName?: () => void
	onclickVersion?: () => void
	onclickDownload?: () => void
}

interface Props {
	projectV3: Labrinth.Projects.v3.Project | null
	tags: {
		gameVersions: GameVersionTag[]
		loaders: PlatformTag[]
	}
	requiredContent?: RequiredContent | null
	recommendedVersion?: string | null
	supportedVersions?: string[]
	loaders?: string[]
	ping?: number
}

const props = withDefaults(defineProps<Props>(), {
	requiredContent: null,
	recommendedVersion: null,
	supportedVersions: () => [],
	loaders: () => [],
	ping: undefined,
})

const ipAddress = computed(() => props.projectV3?.minecraft_java_server?.address ?? '')
const languages = computed(() => props.projectV3?.minecraft_server?.languages ?? [])
const country = computed(() => props.projectV3?.minecraft_server?.country)

const recommendedVersions = computed(() => {
	if (props.recommendedVersion) return [props.recommendedVersion]

	const content = props.projectV3?.minecraft_java_server?.content
	if (content?.kind === 'vanilla' && content.recommended_game_version) {
		return [content.recommended_game_version]
	}

	return []
})

const hasContent = computed(
	() =>
		!!ipAddress.value ||
		!!props.requiredContent ||
		recommendedVersions.value.length > 0 ||
		supportedVersionsList.value.length > 0 ||
		languages.value.length > 0 ||
		props.ping !== undefined,
)

const supportedVersionsList = computed(() => {
	if (props.supportedVersions.length > 0) return props.supportedVersions

	const content = props.projectV3?.minecraft_java_server?.content
	if (content?.kind === 'vanilla' && content.supported_game_versions?.length) {
		return content.supported_game_versions.filter((v): v is string => !!v)
	}

	return []
})

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

function handleCopyIP() {
	navigator.clipboard.writeText(ipAddress.value).then(() => {
		addNotification({
			type: 'success',
			title: formatMessage(messages.copied),
			text: formatMessage(messages.copiedText),
		})
	})
}

const messages = defineMessages({
	copied: {
		id: `project.about.server.copied`,
		defaultMessage: 'Copied!',
	},
	copiedText: {
		id: `project.about.server.copiedText`,
		defaultMessage: 'IP address copied to clipboard',
	},
	title: {
		id: `project.about.server.title`,
		defaultMessage: 'Server details',
	},
	latency: {
		id: `project.about.server.latency`,
		defaultMessage: 'Latency',
	},
})
</script>
