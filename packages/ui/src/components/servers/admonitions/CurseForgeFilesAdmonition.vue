<script setup lang="ts">
import { RightArrowIcon } from '@modrinth/assets'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

withDefaults(
	defineProps<{
		fileCount: number
		dismissible?: boolean
	}>(),
	{ dismissible: true },
)

const emit = defineEmits<{
	review: []
	dismiss: []
}>()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	title: {
		id: 'servers.admonitions.curseforge-files.title',
		defaultMessage: 'Review where these files should be installed',
	},
	body: {
		id: 'servers.admonitions.curseforge-files.body',
		defaultMessage:
			'We couldn’t determine where {count, plural, one {# file} other {# files}} from your modpack should be installed, so we’ve enabled them for both the server and players. Review them, as files that don’t belong on the server may prevent it from starting.',
	},
	review: {
		id: 'servers.admonitions.curseforge-files.review',
		defaultMessage: 'Review files',
	},
})
</script>

<template>
	<Admonition
		type="circle-warning"
		:header="formatMessage(messages.title)"
		:dismissible="dismissible"
		class="gap-x-3"
		@dismiss="emit('dismiss')"
	>
		<span class="leading-6">{{ formatMessage(messages.body, { count: fileCount }) }}</span>
		<template #actions>
			<Button type="colored" color="orange" size="lg" @click="emit('review')">
				{{ formatMessage(messages.review) }}
				<RightArrowIcon aria-hidden="true" />
			</Button>
		</template>
	</Admonition>
</template>
