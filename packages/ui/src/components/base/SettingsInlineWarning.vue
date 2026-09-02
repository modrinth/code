<script setup lang="ts">
import { TriangleAlertIcon } from '@modrinth/assets'
</script>

<template>
	<div class="flex flex-wrap items-center gap-1.5 text-orange">
		<TriangleAlertIcon aria-hidden="true" class="size-4 shrink-0" />
		<div class="min-w-0">
			<slot />
		</div>
	</div>
</template>
