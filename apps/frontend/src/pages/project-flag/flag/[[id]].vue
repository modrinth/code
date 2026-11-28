<template>
	<div />
</template>

<script setup lang="ts">
definePageMeta({
	layout: 'empty',
	middleware: [
		'auth',
		'staff',
		(to) => {
			const { project, ...query } = to.query
			return navigateTo(
				{
					path: '/moderation/project-flag',
					query: { ...query, rule: String(to.params.id || 'new'), ruleProject: project },
					hash: to.hash,
				},
				{ replace: true },
			)
		},
	],
	validate: (route) => !route.params.id || /^[A-Za-z0-9]+$/.test(String(route.params.id)),
})
</script>
