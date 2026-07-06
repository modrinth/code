<template>
	<Avatar
		:src="src"
		:alt="alt"
		:size="size"
		:tint-by="tintBy"
		:circle="circle"
		:no-shadow="noShadow"
		:raised="raised"
		:loading="loading"
		v-bind="$attrs"
	/>
</template>

<script setup lang="ts">
import Avatar from '../Avatar.vue'

withDefaults(
	defineProps<{
		src?: string | null
		alt: string
		size?: string
		tintBy?: string | null
		circle?: boolean
		noShadow?: boolean
		raised?: boolean
		loading?: 'eager' | 'lazy'
	}>(),
	{
		src: null,
		size: '64px',
		tintBy: null,
		circle: false,
		noShadow: false,
		raised: false,
		loading: 'eager',
	},
)
</script>
