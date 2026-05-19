<template>
	<BaseEvent>
		Changed modpack
		<span v-if="newVersionId" class="text-secondary">
			to version <span class="font-mono">{{ shortVersionId }}</span>
		</span>
	</BaseEvent>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import BaseEvent from './BaseEvent.vue'

const props = defineProps<{
	newVersionId?: string | null
}>()

const shortVersionId = computed(() =>
	props.newVersionId && props.newVersionId.length > 12
		? props.newVersionId.slice(0, 8)
		: props.newVersionId,
)
</script>
