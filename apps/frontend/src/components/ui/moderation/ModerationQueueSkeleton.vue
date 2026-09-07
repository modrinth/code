<template>
	<div class="flex flex-col gap-3">
		<div
			v-for="i in 3"
			:key="`loading-skeleton-${i}`"
			class="flex h-[98px] w-full animate-pulse rounded-2xl bg-surface-3"
		></div>
	</div>
</template>
