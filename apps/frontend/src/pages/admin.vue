<script setup lang="ts">
definePageMeta({
	middleware: ['auth', 'staff'],
})

useSeoMeta({
	robots: 'noindex',
})
</script>
<template>
	<NuxtPage />
</template>
