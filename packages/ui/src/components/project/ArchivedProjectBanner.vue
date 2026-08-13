<script setup lang="ts">
import { ArchiveIcon } from '@modrinth/assets'
import { computed } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

const props = defineProps<{
	title: string
	reason?: string | null
}>()

const { formatMessage } = useVIntl()

const trimmedReason = computed(() => props.reason?.trim() || null)

const messages = defineMessages({
	header: {
		id: 'project.status.archived.header',
		defaultMessage: '{title} has been archived',
	},
	body: {
		id: 'project.status.archived.body',
		defaultMessage:
			'{title} will not receive any further updates unless the author decides to unarchive the project.',
	},
})
</script>

<template>
	<Admonition type="info" :header="formatMessage(messages.header, { title })">
		<template #icon="{ iconClass }">
			<ArchiveIcon :class="iconClass" />
		</template>
		<span>{{ trimmedReason ?? formatMessage(messages.body, { title }) }}</span>
	</Admonition>
</template>
