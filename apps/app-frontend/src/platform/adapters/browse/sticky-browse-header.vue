<template>
	<div
		ref="root"
		class="sticky top-0 z-20 -mx-6 -mt-6 rounded-tl-[--radius-xl] border-0 border-solid bg-surface-1 px-6 pt-6 border-surface-5"
		:class="{ 'border-t': isStuck }"
	>
		<slot />
		<div
			aria-hidden="true"
			class="mt-4 h-px bg-surface-5 transition-[margin] duration-200 ease-out motion-reduce:transition-none"
			:class="{ '-mx-6': isStuck }"
		/>
	</div>
</template>

<script setup lang="ts">
import { useStickyObserver } from '@modrinth/ui'
import { ref } from 'vue'

const root = ref<HTMLElement | null>(null)
const { isStuck } = useStickyObserver(root, 'StickyBrowseHeader')
</script>
