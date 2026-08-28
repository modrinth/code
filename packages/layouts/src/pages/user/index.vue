<script setup lang="ts">
import { injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'

import Sidebar from '../../components/Sidebar.vue'
import SidebarCard from '../../components/SidebarCard.vue'

const props = defineProps<{
	userId: string
}>()

const api = injectModrinthClient()
const { data: user } = useQuery({
	queryKey: ['user', props.userId],
	queryFn: () => api.labrinth.users_v3.get(props.userId),
	enabled: !!props.userId,
})
</script>
<template>
	<div class="flex flex-col gap-2">
		<img :src="user?.avatar_url" class="size-10 rounded-full" />
		<div>Hello user {{ user?.username }}!</div>
	</div>
	<Sidebar>
		<SidebarCard> Sidebar stuff!</SidebarCard>
		<SidebarCard> the stuff in the sidebar goes here</SidebarCard>
	</Sidebar>
</template>
