<script setup lang="ts">
definePageMeta({
	layout: 'empty',
})

useSeoMeta({
	robots: 'noindex',
})

const route = useRoute()

const deepLink = computed(() => {
	const inviteId = encodeURIComponent(route.params.inviteId as string)
	return `modrinth://share/${inviteId}`
})
</script>

<template>
	<main class="grid min-h-screen place-items-center bg-bg px-4">
		<a :href="deepLink" class="btn btn-primary">Open in Modrinth App</a>
	</main>
</template>
