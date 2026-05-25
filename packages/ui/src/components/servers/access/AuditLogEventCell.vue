<template>
	<div class="audit-log-table-event min-w-0 truncate">
		<component :is="event.component" v-bind="event.props" class="audit-log-table-event-component" />
	</div>
</template>

<script setup lang="ts">
import type { ParsedAuditEvent } from './events/types'

defineProps<{
	event: ParsedAuditEvent
}>()
</script>
