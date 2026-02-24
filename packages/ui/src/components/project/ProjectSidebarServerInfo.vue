<template>
	<div class="flex flex-col gap-3">
		<div
			v-if="ipAddress"
			class="bg-surface-4 flex gap-2 justify-between rounded-2xl items-center px-3 pr-2 h-12"
		>
			<div v-tooltip="`Java IP: ${ipAddress}`" class="font-semibold text-contrast truncate">
				{{ ipAddress }}
			</div>

			<ButtonStyled type="transparent" size="small">
				<button v-tooltip="'Copy IP address to clipboard'" @click="handleCopyIP">
					<CopyIcon />
				</button>
			</ButtonStyled>
		</div>

		<section v-if="requiredContent" class="flex flex-col gap-2">
			<h3 class="text-primary text-base m-0">Required content</h3>
			<ServerModpackContent
				:name="requiredContent.name"
				:icon="requiredContent.icon"
				class="text-contrast"
				:onclick="requiredContent?.onclick"
			/>
		</section>
		<section v-if="recommendedVersions.length" class="flex flex-col gap-2">
			<h3 class="text-primary text-base m-0">
				<template v-if="supportedVersionsList.length"> Recommended version </template>
				<template v-else>Version</template>
			</h3>
			<div class="flex flex-wrap gap-1.5">
				<TagItem
					v-for="version in formatVersionsForDisplay(recommendedVersions, tags.gameVersions)"
					:key="`recommended-tag-${version}`"
				>
					{{ version }}
				</TagItem>
			</div>
		</section>
		<section v-if="supportedVersionsList.length" class="flex flex-col gap-2">
			<h3 class="text-primary text-base m-0">Supported versions</h3>
			<div class="flex flex-wrap gap-1.5">
				<TagItem
					v-for="version in formatVersionsForDisplay(supportedVersionsList, tags.gameVersions)"
					:key="`supported-tag-${version}`"
				>
					{{ version }}
				</TagItem>
			</div>
		</section>
	</div>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CopyIcon } from '@modrinth/assets'
import { formatVersionsForDisplay, type GameVersionTag, type PlatformTag } from '@modrinth/utils'
import { computed } from 'vue'

import { injectNotificationManager } from '../../providers'
import ButtonStyled from '../base/ButtonStyled.vue'
import TagItem from '../base/TagItem.vue'
import ServerModpackContent from './server/ServerModpackContent.vue'

interface RequiredContent {
	name: string
	icon?: string
	onclick?: () => void
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
}

const props = withDefaults(defineProps<Props>(), {
	requiredContent: null,
	recommendedVersion: null,
	supportedVersions: () => [],
})

const ipAddress = computed(() => props.projectV3?.minecraft_java_server?.address ?? '')

const recommendedVersions = computed(() => {
	if (props.recommendedVersion) return [props.recommendedVersion]

	const content = props.projectV3?.minecraft_java_server?.content
	if (content?.kind === 'vanilla' && content.recommended_game_version) {
		return [content.recommended_game_version]
	}

	return []
})

const supportedVersionsList = computed(() => {
	if (props.supportedVersions.length > 0) return props.supportedVersions

	const content = props.projectV3?.minecraft_java_server?.content
	if (content?.kind === 'vanilla' && content.supported_game_versions?.length) {
		return content.supported_game_versions.filter((v): v is string => !!v)
	}

	return []
})

const { addNotification } = injectNotificationManager()

function handleCopyIP() {
	navigator.clipboard.writeText(ipAddress.value).then(() => {
		addNotification({
			type: 'success',
			title: 'Copied!',
			text: 'IP address copied to clipboard',
		})
	})
}
</script>
