<script setup lang="ts">
import ModerationAppearance from '~/components/ui/moderation/settings/ModerationAppearance.vue'
import ModerationSettings from '~/components/ui/moderation/settings/ModerationSettings.vue'

definePageMeta({
	middleware: ['auth', 'staff'],
})

useSeoMeta({
	robots: 'noindex',
})
</script>

<template>
	<div class="flex flex-col gap-4">
		<ModerationSettings section="public-pages" />
		<ModerationSettings section="general" />
		<ModerationAppearance />
	</div>
</template>
