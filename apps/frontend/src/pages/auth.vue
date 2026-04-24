<script setup lang="ts">
definePageMeta({
	middleware: ['launcher-auth'],
})

useSeoMeta({
	robots: 'noindex',
})
</script>
<template>
	<div class="grid min-h-[calc(100vh-4.5rem)] place-items-center pb-20">
		<NuxtPage />
	</div>
</template>
