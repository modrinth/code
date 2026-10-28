<script setup lang="ts">
definePageMeta({
	middleware: ['launcher-auth'],
})

useSeoMeta({
	robots: 'noindex',
})
</script>
<template>
	<div class="grid min-h-[calc(100vh-14.5rem)] place-items-center px-2.5 pb-20 pt-10">
		<NuxtPage />
	</div>
</template>
