<script setup lang="ts">
definePageMeta({
	key: 'discover-type',
})

const route = useRoute()

if (!route.params.type || typeof route.params.type !== 'string') {
	throw createError({
		statusCode: 404,
	})
}
</script>
<template>
	<NuxtPage />
</template>
