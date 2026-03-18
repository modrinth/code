<template>
	<div
		v-if="subdomain && !isHidden"
		v-tooltip="'Copy custom URL'"
		class="flex min-w-0 flex-row items-center gap-2 truncate hover:cursor-pointer"
	>
		<Separator v-if="!noSeparator" />

		<div class="flex flex-row items-center gap-1.5">
			<LinkIcon />
			<div
				class="flex min-w-0 text-sm font-semibold"
				:class="serverId ? 'hover:underline' : ''"
				@click="copySubdomain"
			>
				{{ subdomain }}.modrinth.gg
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { LinkIcon } from '@modrinth/assets'
import { injectNotificationManager } from '@modrinth/ui'
import { useStorage } from '@vueuse/core'
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import Separator from './Separator.vue'

const { addNotification } = injectNotificationManager()

const props = defineProps<{
	subdomain: string
	noSeparator?: boolean
}>()

const copySubdomain = () => {
	navigator.clipboard.writeText(props.subdomain + '.modrinth.gg')
	addNotification({
		title: 'Custom URL copied',
		text: "Your server's URL has been copied to your clipboard.",
		type: 'success',
	})
}

const route = useRoute()
const serverId = computed(() => route.params.id as string)

const userPreferences = useStorage(`pyro-server-${serverId.value}-preferences`, {
	hideSubdomainLabel: false,
})

const isHidden = computed(() => userPreferences.value.hideSubdomainLabel)
</script>
