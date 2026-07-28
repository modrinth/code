<script setup lang="ts">
import { useRootBreadcrumb } from '@/providers/breadcrumbs'

useRootBreadcrumb({
	slot: 'root',
	id: 'worlds',
	label: 'Worlds',
	to: '/worlds',
})
</script>
<template>
	<div class="p-6 flex flex-col gap-2">Worlds</div>
</template>
