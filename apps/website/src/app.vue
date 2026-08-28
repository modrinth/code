<script setup lang="ts">
import { NuxtModrinthClient } from '@modrinth/api-client'
import { provideModrinthClient, providePageContext } from '@modrinth/ui'

provideModrinthClient(
	new NuxtModrinthClient({
		labrinthBaseUrl: 'https://api.modrinth.com',
	}),
)

providePageContext({
	hierarchicalSidebarAvailable: ref(false),
	showAds: ref(false),
	adConsentAvailable: ref(false),
	floatingActionBarOffsets: {
		left: ref(0),
		right: ref(0),
	},
})
</script>
<template>
	<div class="page-layout">
		<NuxtPage />
	</div>
</template>
<style scoped>
.page-layout {
	@apply max-w-[min(80rem,100%-1.5rem)] mx-auto h-[100dvh] pt-6;

	&:has(#page-sidebar:not(:empty)) {
		@apply grid grid-cols-[1fr_300px];
		grid-template-areas: 'anything sidebar';
	}
}
</style>
<style>
#page-sidebar {
	@apply pl-4 border-l border-surface-5;
	grid-area: sidebar;
}
</style>
