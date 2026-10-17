<template><div /></template>

<script setup lang="ts">
/** The archive viewer moved to its own full-screen page; links to the old tab still open it. */
definePageMeta({
	middleware: [
		'auth',
		'staff',
		(to) =>
			navigateTo(
				{ path: '/archive', query: { ...to.query, project: String(to.params.project) } },
				{ replace: true },
			),
	],
})
</script>
