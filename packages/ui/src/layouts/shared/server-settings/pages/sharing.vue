<script setup lang="ts">
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useServerSharingSettings } from '#ui/layouts/shared/server-sharing'
import { provideSharingSettings, SharingSettingsLayout } from '#ui/layouts/shared/sharing-settings'

import { injectServerSettings } from '../providers/server-settings'

const { siteUrl } = injectServerSettings()
const { formatMessage } = useVIntl()
const { settings, canSetup, permissionDeniedMessage, sharedInstanceId } = useServerSharingSettings()
provideSharingSettings(settings)

const messages = defineMessages({
	unpublishDescription: {
		id: 'server.settings.sharing.unpublish-description',
		defaultMessage:
			'Remove the shared instance for this server from Modrinth and stop sending content updates to players. Your server and its content will not be affected.',
	},
	unpublishConfirmation: {
		id: 'server.settings.sharing.unpublish-confirmation',
		defaultMessage:
			'This deletes the shared instance and disables its invite links. Players will stop receiving shared content updates. Your server and its content will remain available.',
	},
	notShared: {
		id: 'server.settings.sharing.not-shared',
		defaultMessage:
			'This world is not shared. Use Play server or Invite players on the Play page to publish its content.',
	},
})
</script>

<template>
	<p v-if="!canSetup" class="m-0 text-secondary">{{ permissionDeniedMessage }}</p>
	<p v-else-if="!sharedInstanceId" class="m-0 text-secondary">
		{{ formatMessage(messages.notShared) }}
	</p>
	<SharingSettingsLayout
		v-else
		:site-url="siteUrl"
		:unpublish-description="messages.unpublishDescription"
		:unpublish-confirmation="messages.unpublishConfirmation"
	/>
</template>
