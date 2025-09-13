<script setup lang="ts">
import BatchScanProgressAlert, {
	type BatchScanProgress,
} from '@/components/ui/moderation/delphi/BatchScanProgressAlert.vue'

// TODO: Live way to update this via the backend, polling?
const batchScanProgressInformation = computed<BatchScanProgress | undefined>(() => {
	return {
		total: 58,
		complete: 20,
	}
})
</script>

<template>
	<BatchScanProgressAlert
		v-if="batchScanProgressInformation"
		:progress="batchScanProgressInformation"
	/>
</template>
