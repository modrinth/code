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

import { injectModrinthClient } from '../../../providers/api-client'
import MarkdownEmbedCard from './MarkdownEmbedCard.vue'

const props = defineProps<{
	id: string
}>()

const client = injectModrinthClient()

const { data: user, isLoading } = useQuery({
	queryKey: ['markdown-embed-user', () => props.id],
	queryFn: () => client.labrinth.users_v3.get(props.id),
	enabled: () => !!props.id,
	retry: false,
})
</script>
