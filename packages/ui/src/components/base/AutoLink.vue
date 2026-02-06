<template>
	<router-link
		v-if="typeof to === 'object' && (to?.path || to?.query || to?.startsWith('/'))"
		:to="to"
		v-bind="$attrs"
	>
		<slot />
	</router-link>
	<a v-else-if="typeof to === 'string' && to?.startsWith('http')" :href="to" v-bind="$attrs">
		<slot />
	</a>
	<button v-else-if="typeof to === 'function'" v-bind="$attrs" class="inline bg-transparent border-none p-0 m-0 cursor-pointer" @click="to()">
		<slot />
	</button>
	<span v-else v-bind="$attrs">
		<slot />
	</span>
</template>

<script setup lang="ts">
defineProps<{
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	to: any
}>()

defineOptions({
	inheritAttrs: false,
})
</script>
