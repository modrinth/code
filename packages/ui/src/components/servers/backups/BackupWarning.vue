<script setup lang="ts">
import { IssuesIcon } from '@modrinth/assets'

import IntlFormatted from '#ui/components/base/IntlFormatted.vue'
import { defineMessages } from '#ui/composables/i18n'

import AutoLink from '../../base/AutoLink.vue'

defineProps<{
	backupLink: string
}>()

const messages = defineMessages({
	body: {
		id: 'servers.backups.warning.body',
		defaultMessage:
			'You may want to <create-link>create a backup</create-link> before proceeding, as this process is irreversible and may permanently alter your world or the files on your server.',
	},
})
</script>
<template>
	<div
		class="flex gap-3 rounded-2xl border-2 border-solid border-orange bg-bg-orange px-4 py-3 font-medium text-contrast"
	>
		<IssuesIcon class="mt-1 h-5 w-5 shrink-0 text-orange" />
		<span class="leading-normal">
			<IntlFormatted :message-id="messages.body">
				<template #create-link="{ children }">
					<AutoLink
						:to="backupLink"
						class="font-semibold text-orange hover:underline active:brightness-125"
					>
						<component :is="() => children" />
					</AutoLink>
				</template>
			</IntlFormatted>
		</span>
	</div>
</template>
