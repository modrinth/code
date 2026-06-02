<template>
	<div class="flex flex-wrap items-center gap-x-1.5 gap-y-1 leading-snug text-primary">
		<button
			v-if="inviterName"
			type="button"
			class="inline-flex min-w-0 items-center border-0 bg-transparent p-0 font-semibold text-contrast hover:underline"
			@click="openInviterProfile(inviterName)"
		>
			<Avatar
				:src="inviterAvatarUrl"
				:alt="inviterName"
				circle
				size="xxs"
				no-shadow
				class="mr-1.5 inline-flex"
			/>
			<span>{{ inviterName }}</span>
		</button>
		<span>
			<span v-if="inviterName" class="whitespace-nowrap">has invited you to manage</span>
			<span v-else class="whitespace-nowrap">You have been invited to manage</span>
			<span class="font-semibold text-contrast ml-1">{{ serverName }}</span>
			<span>.</span>
		</span>
	</div>
</template>

<script setup>
import { Avatar } from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'

import { config } from '@/config'

defineProps({
	inviterName: {
		type: String,
		default: null,
	},
	inviterAvatarUrl: {
		type: String,
		default: null,
	},
	serverName: {
		type: String,
		required: true,
	},
})

function openInviterProfile(username) {
	openUrl(`${config.siteUrl}/user/${encodeURIComponent(username)}`)
}
</script>
