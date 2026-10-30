<script setup lang="ts">
import { useUpdateAllTruncatedTooltip } from './use-update-all-truncated-tooltip'

const props = defineProps<{ version: string }>()
const { element, tooltip } = useUpdateAllTruncatedTooltip(() => props.version)
</script>

<template>
	<span ref="element" v-tooltip="tooltip" class="block min-w-0 truncate">{{ version }}</span>
</template>
