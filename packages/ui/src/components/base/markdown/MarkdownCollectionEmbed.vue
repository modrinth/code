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
import { computed, onServerPrefetch } from 'vue'

import { injectModrinthClient } from '#ui/providers'
import MarkdownEmbedCard from './MarkdownEmbedCard.vue'

const props = defineProps<{
	id: string
}>()

const client = injectModrinthClient()

const query = useQuery({
	queryKey: computed(() => ['markdown-embed-collection', props.id]),
	queryFn: () => client.labrinth.collections.get(props.id),
	enabled: () => !!props.id,
	retry: false,
})
const { data: collection, isLoading } = query

onServerPrefetch(() => query.suspense())
</script>
