<template>
	<div class="flex h-full items-center justify-center p-3 text-secondary">
		<span v-if="params.group?.api.location.type === 'popout'" role="status">
			No tabs open in this panel
		</span>
	</div>
</template>

<script setup lang="ts">
import type { WatermarkRendererInitParameters } from 'dockview-vue'

defineProps<{ params: WatermarkRendererInitParameters }>()
</script>
