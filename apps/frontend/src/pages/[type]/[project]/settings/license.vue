<script setup lang="ts">
const route = useRoute()
await navigateTo(
	{
		path: route.path.replace(/\/license\/?$/, ''),
		hash: route.hash || '#license',
		query: route.query,
	},
	{ replace: true, redirectCode: 301 },
)
</script>

<template><div /></template>
