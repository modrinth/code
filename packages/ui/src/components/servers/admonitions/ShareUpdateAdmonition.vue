<script setup lang="ts">
import { SpinnerIcon, UploadIcon } from '@modrinth/assets'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

defineProps<{ disabled: boolean; publishing: boolean; reviewing: boolean }>()
const emit = defineEmits<{ review: [] }>()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	unpublished: {
		id: 'app.instance.admonitions.shared-instance.changes-header',
		defaultMessage: "Your changes haven't been shared yet",
	},
	unpublishedBody: {
		id: 'servers.play.unpublished-body',
		defaultMessage: 'Push an update to share your server’s content changes with players.',
	},
	pushUpdate: {
		id: 'app.instance.admonitions.shared-instance.publish-button',
		defaultMessage: 'Push update',
	},
	publishing: {
		id: 'app.instance.admonitions.shared-instance.publishing-button',
		defaultMessage: 'Pushing...',
	},
	reviewing: {
		id: 'app.instance.admonitions.shared-instance.reviewing-button',
		defaultMessage: 'Reviewing...',
	},
})
</script>

<template>
	<Admonition type="info" :header="formatMessage(messages.unpublished)" inline-actions>
		{{ formatMessage(messages.unpublishedBody) }}
		<template #actions>
			<Button type="colored" color="blue" size="lg" :disabled="disabled" @click="emit('review')">
				<SpinnerIcon v-if="publishing || reviewing" class="animate-spin" aria-hidden="true" />
				<UploadIcon v-else aria-hidden="true" />
				{{
					formatMessage(
						publishing ? messages.publishing : reviewing ? messages.reviewing : messages.pushUpdate,
					)
				}}
			</Button>
		</template>
	</Admonition>
</template>
