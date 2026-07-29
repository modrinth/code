<script setup lang="ts">
import type { VersionChannel } from '@modrinth/utils'
import { computed } from 'vue'

import { commonMessages } from '#ui/utils/common-messages.ts'

import { useVIntl } from '../../composables/i18n'
import TagItem from '../base/TagItem.vue'

const { formatMessage } = useVIntl()

const props = defineProps<{
	channel: VersionChannel
}>()

const RELEASE_CHANNELS = {
	release: {
		message: commonMessages.release,
		bgColor: 'var(--color-green-bg)',
		color: 'var(--color-green)',
	},
	beta: {
		message: commonMessages.beta,
		bgColor: 'var(--color-orange-bg)',
		color: 'var(--color-orange)',
	},
	alpha: {
		message: commonMessages.alpha,
		bgColor: 'var(--color-red-bg)',
		color: 'var(--color-red)',
	},
}

const releaseChannel = computed(() => RELEASE_CHANNELS[props.channel])
</script>

<template>
	<TagItem
		class="py-1.5"
		:style="{ '--_bg-color': releaseChannel.bgColor, '--_color': releaseChannel.color }"
	>
		{{ formatMessage(releaseChannel.message) }}
	</TagItem>
</template>
