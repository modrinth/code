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

import { injectModrinthClient } from '../../../providers/api-client'
import MarkdownEmbedCard from './MarkdownEmbedCard.vue'

const props = defineProps<{
	id: string
}>()

const client = injectModrinthClient()

const { data: organization, isLoading } = useQuery({
	queryKey: ['markdown-embed-organization', () => props.id],
	queryFn: () => client.labrinth.organizations_v3.get(props.id),
	enabled: () => !!props.id,
	retry: false,
})
</script>
