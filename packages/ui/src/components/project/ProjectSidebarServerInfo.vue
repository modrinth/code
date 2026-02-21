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
			<ServerRequiredContent :name="requiredContent.name" :icon="requiredContent.icon" />
		</section>
		<section v-if="versions.length" class="flex flex-col gap-2">
			<h3 class="text-primary text-base m-0">Versions</h3>
			<div class="flex flex-wrap gap-1.5">
				<TagItem
					v-for="version in formatVersionsForDisplay(versions, tags.gameVersions)"
					:key="`version-tag-${version}`"
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
import ServerRequiredContent from './server/ServerRequiredContent.vue'

interface RequiredContent {
	name: string
	icon?: string
}

interface Props {
	projectV3: Labrinth.Projects.v3.Project | null
	tags: {
		gameVersions: GameVersionTag[]
		loaders: PlatformTag[]
	}
	requiredContent?: RequiredContent | null
	serverGameVersions?: string[]
}

const props = withDefaults(defineProps<Props>(), {
	requiredContent: null,
	serverGameVersions: () => [],
})

const ipAddress = computed(() => props.projectV3?.minecraft_java_server?.address ?? '')

const versions = computed(() => {
	if (props.serverGameVersions.length > 0) return props.serverGameVersions

	const content = props.projectV3?.minecraft_java_server?.content
	if (!content) return []

	if (content.kind !== 'vanilla') return []

	const allVersions = [content.recommended_game_version, ...(content.supported_game_versions ?? [])]
	return Array.from(new Set(allVersions.filter((v): v is string => !!v)))
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
