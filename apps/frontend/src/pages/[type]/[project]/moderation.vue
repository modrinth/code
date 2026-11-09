<script setup lang="ts">
const route = useRoute()
await navigateTo(
	{
		path: `/${route.params.type}/${route.params.project}/settings`,
		query: { ...route.query, moderation: 'thread' },
		hash: route.hash,
	},
	{ replace: true },
)
</script>

<template><div /></template>
