<script setup lang="ts">
import ModerationKeybinds from '~/components/ui/moderation/settings/ModerationKeybinds.vue'

definePageMeta({
	middleware: ['auth', 'staff'],
})

useSeoMeta({
	robots: 'noindex',
})
</script>

<template>
	<ModerationKeybinds />
</template>
