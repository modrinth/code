<script setup lang="ts">
import { injectModrinthClient } from '@modrinth/ui'

import { getAuthUrl } from '~/composables/auth.js'

definePageMeta({
	layout: 'empty',
	middleware: 'auth',
})

const route = useRoute()
const auth = await useAuth()
const client = injectModrinthClient()
const error = ref<unknown>(null)
const isLinkedCallback = computed(() => route.query.callback === 'linked')

onMounted(async () => {
	if (isLinkedCallback.value) return

	try {
		if (!auth.value.user?.auth_providers?.includes('discord')) {
			window.location.href = `${getAuthUrl('discord', '/discord/link')}&token=${auth.value.token}`
			return
		}

		const res = await client.labrinth.auth_internal.createDiscordCommunityLink()
		window.location.href = res.url
	} catch (err) {
		error.value = err
	}
})
</script>

<template>
	<section class="discord-link-container universal-card">
		<h1>{{ isLinkedCallback ? 'Modrinth account linked' : 'Linking Discord' }}</h1>
		<p v-if="isLinkedCallback">Your Modrinth account has been linked to the Discord server.</p>
		<p v-else-if="!error">Connecting your Modrinth account to the Discord server...</p>
		<p v-else>Discord linking failed. Please try again later.</p>
	</section>
</template>

<style scoped>
.discord-link-container {
	width: 26rem;
	max-width: calc(100% - 2rem);
	margin: 1rem auto;
	display: flex;
	flex-direction: column;
	gap: 2rem;
}

.discord-link-container h1 {
	font-size: var(--font-size-xl);
	margin: 0 0 -1rem 0;
	color: var(--color-contrast);
}

.discord-link-container p {
	margin: 0;
}
</style>
