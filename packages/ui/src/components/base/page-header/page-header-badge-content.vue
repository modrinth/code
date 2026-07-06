<template>
	<component :is="badge.icon" v-if="badge.icon" aria-hidden="true" v-bind="badge.iconProps" />
	<span v-if="badge.label">{{ badge.label }}</span>
</template>

<script setup lang="ts">
import type { PageHeaderBadgeContent } from './types'

defineProps<{
	badge: PageHeaderBadgeContent
}>()
</script>
