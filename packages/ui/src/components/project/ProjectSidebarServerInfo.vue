<template>
	<div class="flex flex-col gap-3">
		<div class="bg-surface-4 flex gap-2 justify-between rounded-2xl items-center px-3 pr-2 h-12">
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
			<div class="flex flex-wrap gap-1">
				<Avatar :src="requiredContent.icon" size="24px" />
				<span>
					{{ requiredContent.name }}
				</span>
			</div>
		</section>
		<section class="flex flex-col gap-2">
			<h3 class="text-primary text-base m-0">Versions</h3>
			<div class="flex flex-wrap gap-1">
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
import { CopyIcon } from '@modrinth/assets'
import { formatVersionsForDisplay, type GameVersionTag, type PlatformTag } from '@modrinth/utils'

import { useVIntl } from '../../composables/i18n'
import { injectNotificationManager } from '../../providers'
import Avatar from '../base/Avatar.vue'
import ButtonStyled from '../base/ButtonStyled.vue'
import TagItem from '../base/TagItem.vue'

interface RequiredContent {
	name: string
	icon: string
}

interface Props {
	ipAddress: string
	requiredContent: RequiredContent
	versions: string[]
	tags: {
		gameVersions: GameVersionTag[]
		loaders: PlatformTag[]
	}
}

const { ipAddress, requiredContent, versions, tags } = defineProps<Props>()

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()

function handleCopyIP() {
	// Implement the logic to copy the IP address here
	navigator.clipboard.writeText(ipAddress).then(() => {
		// Optionally, show a success message or tooltip
		addNotification({
			type: 'success',
			title: 'Copied!',
			text: 'IP address copied to clipboard',
		})
	})
}
</script>
