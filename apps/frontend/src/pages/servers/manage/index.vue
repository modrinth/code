<script setup lang="ts">
import { ServersManagePageIndex } from '@modrinth/ui'

definePageMeta({
	middleware: 'auth',
})

useHead({
	title: 'Servers - Modrinth',
})
</script>

<template>
	<ServersManagePageIndex />
</template>
