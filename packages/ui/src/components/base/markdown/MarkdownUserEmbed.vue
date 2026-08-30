<template>
	<MarkdownEmbedCard
		:to="user ? `/user/${user.username}` : ''"
		:icon-url="user?.avatar_url"
		:title="user?.username"
		:description="user?.bio"
		:loading="isLoading"
		:not-found="!isLoading && !user"
		not-found-message="User not found"
	/>
</template>

<script setup lang="ts">
import { useQuery } from '@tanstack/vue-query'
import { computed, onServerPrefetch } from 'vue'

import { injectModrinthClient } from '#ui/providers'
import MarkdownEmbedCard from './MarkdownEmbedCard.vue'

const props = defineProps<{
	id: string
}>()

const client = injectModrinthClient()

const query = useQuery({
	queryKey: computed(() => ['markdown-embed-user', props.id]),
	queryFn: () => client.labrinth.users_v3.get(props.id),
	enabled: () => !!props.id,
	retry: false,
})
const { data: user, isLoading } = query

onServerPrefetch(() => query.suspense())
</script>
