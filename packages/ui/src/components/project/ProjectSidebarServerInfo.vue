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
			<div class="flex gap-1.5 items-center">
				<Avatar :src="requiredContent.icon" size="24px" />
				<span class="truncate font-medium text-contrast">
					{{ requiredContent.name }}
				</span>
			</div>
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
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { injectNotificationManager } from '../../providers'
import { injectModrinthClient } from '../../providers/api-client'
import Avatar from '../base/Avatar.vue'
import ButtonStyled from '../base/ButtonStyled.vue'
import TagItem from '../base/TagItem.vue'

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
}

const props = defineProps<Props>()
const { labrinth } = injectModrinthClient()

const ipAddress = computed(() => props.projectV3?.minecraft_java_server?.address ?? '')

const versions = computed(() => {
	const content = props.projectV3?.minecraft_java_server?.content
	if (!content) return []

	if (content.kind !== 'vanilla') {
		return modpackVersion.value?.game_versions ?? []
	}

	const allVersions = [content.recommended_game_version, ...(content.supported_game_versions ?? [])]
	return Array.from(new Set(allVersions.filter((v): v is string => !!v)))
})

const modpackVersionId = computed(() => {
	const content = props.projectV3?.minecraft_java_server?.content
	return content?.kind === 'modpack' ? content.version_id : null
})

const { data: modpackVersion } = useQuery({
	queryKey: computed(() => ['sidebar-modpack-version', modpackVersionId.value] as const),
	queryFn: () => labrinth.versions_v3.getVersion(modpackVersionId.value!),
	enabled: computed(() => !!modpackVersionId.value),
})

const modpackProjectId = computed(() => modpackVersion.value?.project_id ?? null)

const { data: modpackProject } = useQuery({
	queryKey: computed(() => ['sidebar-modpack-project', modpackProjectId.value] as const),
	queryFn: () => labrinth.projects_v3.get(modpackProjectId.value!),
	enabled: computed(() => !!modpackProjectId.value),
})

const requiredContent = computed<RequiredContent | null>(() => {
	if (!modpackProject.value) return null
	return {
		name: modpackProject.value.name,
		icon: modpackProject.value.icon_url,
	}
})

const { addNotification } = injectNotificationManager()

function handleCopyIP() {
	// Implement the logic to copy the IP address here
	navigator.clipboard.writeText(ipAddress.value).then(() => {
		// Optionally, show a success message or tooltip
		addNotification({
			type: 'success',
			title: 'Copied!',
			text: 'IP address copied to clipboard',
		})
	})
}
</script>
