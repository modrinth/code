<template>
	<MarkdownEmbedCard
		:to="collection ? `/collection/${collection.id}` : ''"
		:icon-url="collection?.icon_url"
		:title="collection?.name"
		:description="collection?.description"
		:stat="collection ? `${collection.projects.length} projects` : undefined"
		:loading="isLoading"
		:not-found="!isLoading && !collection"
		not-found-message="Collection not found"
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

const { data: collection, isLoading } = useQuery({
	queryKey: ['markdown-embed-collection', () => props.id],
	queryFn: () => client.labrinth.collections.get(props.id),
	enabled: () => !!props.id,
	retry: false,
})
</script>
