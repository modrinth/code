<script setup lang="ts">
import ModerationKeybinds from '~/components/ui/moderation/settings/ModerationKeybinds.vue'
import ModerationSettings from '~/components/ui/moderation/settings/ModerationSettings.vue'

definePageMeta({
	middleware: ['auth', 'staff'],
})

useSeoMeta({
	robots: 'noindex',
})
</script>

<template>
	<ModerationKeybinds />
	<ModerationSettings />
</template>
