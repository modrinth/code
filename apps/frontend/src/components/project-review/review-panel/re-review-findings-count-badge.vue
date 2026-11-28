<template>
	<span
		v-if="count > 0"
		class="pointer-events-none flex shrink-0 items-center gap-0.5 text-xs font-medium leading-none text-orange"
	>
		<TagCategoryRefreshCcwIcon class="size-3.5 shrink-0 text-orange" aria-hidden="true" />
		<span aria-hidden="true">{{ count }}</span>
		<span class="sr-only">{{
			count === 1 ? `${count} re-review item` : `${count} re-review items`
		}}</span>
	</span>
</template>

<script setup lang="ts">
import { TagCategoryRefreshCcwIcon } from '@modrinth/assets'

defineProps<{ count: number }>()
</script>
