<template>
	<span
		v-if="count > 0"
		class="pointer-events-none flex h-[18px] min-w-[18px] shrink-0 items-center justify-center rounded-full border border-solid border-highlight-orange bg-highlight-orange px-1 text-xs font-medium leading-none text-orange"
	>
		<span aria-hidden="true" class="[text-box:trim-both_cap_alphabetic]">{{ count }}</span>
		<span class="sr-only">{{
			count === 1 ? `${count} selected finding` : `${count} selected findings`
		}}</span>
	</span>
</template>

<script setup lang="ts">
defineProps<{ count: number }>()
</script>
