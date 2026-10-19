<template>
	<div
		:data-review-highlight="enabled ? '' : undefined"
		class="relative flex min-w-0 flex-col gap-[inherit] rounded"
		:class="
			layer === 'behind'
				? [behindClasses, enabled && active ? 'before:opacity-100' : 'before:opacity-0']
				: [overlayClasses, enabled && active ? 'after:opacity-100' : 'after:opacity-0']
		"
	>
		<slot />
	</div>
</template>

<script setup lang="ts">
const behindClasses =
	"isolate before:pointer-events-none before:absolute before:-inset-[6px] before:z-[-1] before:rounded-lg before:bg-[color:color-mix(in_srgb,var(--color-orange)_3%,transparent)] before:content-['']"
const overlayClasses =
	"after:pointer-events-none after:absolute after:-inset-[6px] after:rounded-lg after:bg-[color:color-mix(in_srgb,var(--color-orange)_3%,transparent)] after:content-['']"

withDefaults(defineProps<{ active?: boolean; enabled?: boolean; layer?: 'behind' | 'overlay' }>(), {
	enabled: true,
	layer: 'overlay',
})
</script>
