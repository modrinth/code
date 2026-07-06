<template>
	<Avatar
		:src="src"
		:alt="username"
		:size="size"
		:tint-by="tintBy ?? username"
		circle
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
		username: string
		size?: string
		tintBy?: string | null
		noShadow?: boolean
		raised?: boolean
		loading?: 'eager' | 'lazy'
	}>(),
	{
		src: null,
		size: '96px',
		tintBy: null,
		noShadow: false,
		raised: false,
		loading: 'eager',
	},
)
</script>
