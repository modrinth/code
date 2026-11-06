<script setup lang="ts">
import ModerationSettings from './ModerationSettings.vue'

defineProps<{
	embedded?: boolean
}>()
</script>

<template>
	<ModerationSettings :embedded="embedded" tech-review />
</template>
