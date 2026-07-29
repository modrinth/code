<template>
	<router-link
		v-if="
			(typeof to === 'object' && (to?.path || to?.query)) ||
			(typeof to === 'string' && to?.startsWith('/'))
		"
		:to="to"
		v-bind="$attrs"
		:class="linkClass"
	>
		<slot />
	</router-link>
	<a
		v-else-if="typeof to === 'string' && to?.startsWith('http')"
		:href="to"
		v-bind="$attrs"
		:class="linkClass"
	>
		<slot />
	</a>
	<button
		v-else-if="typeof to === 'function'"
		v-bind="$attrs"
		class="inline bg-transparent border-none p-0 m-0 cursor-pointer"
		:class="linkClass"
		@click="to()"
	>
		<slot />
	</button>
	<span v-else v-bind="$attrs">
		<slot />
	</span>
</template>

<script setup lang="ts">
withDefaults(
	defineProps<{
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		to: any
		linkClass?: string
	}>(),
	{
		linkClass: '',
	},
)

defineOptions({
	inheritAttrs: false,
})
</script>
