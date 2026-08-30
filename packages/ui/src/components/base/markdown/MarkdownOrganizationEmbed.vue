<template>
	<MarkdownEmbedCard
		:to="organization ? `/organization/${organization.slug}` : ''"
		:icon-url="organization?.icon_url"
		:title="organization?.name"
		:description="organization?.description"
		:stat="organization ? `${organization.members.length} members` : undefined"
		:loading="isLoading"
		:not-found="!isLoading && !organization"
		not-found-message="Organization not found"
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
	queryKey: computed(() => ['markdown-embed-organization', props.id]),
	queryFn: () => client.labrinth.organizations_v3.get(props.id),
	enabled: () => !!props.id,
	retry: false,
})
const { data: organization, isLoading } = query

onServerPrefetch(() => query.suspense())
</script>
