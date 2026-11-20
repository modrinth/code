<template>
	<EmptyState :icon="icon" :heading="heading" :description="description">
		<template v-if="$slots.description" #description><slot name="description" /></template>
		<template v-if="$slots.actions" #actions><slot name="actions" /></template>
	</EmptyState>
</template>

<script setup lang="ts">
import { EmptyState } from '@modrinth/ui'
import type { Component } from 'vue'

defineProps<{ icon: Component; heading: string; description?: string }>()
</script>
