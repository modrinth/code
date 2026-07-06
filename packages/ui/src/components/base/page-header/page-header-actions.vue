<template>
	<div v-if="actions.length" class="flex flex-wrap items-center gap-2">
		<template v-for="action in actions" :key="action.id">
			<PageHeaderActionWithPrompt v-if="action.prompt" :action="action" />
			<PageHeaderAction v-else :action="action" />
		</template>
	</div>
</template>

<script setup lang="ts">
import PageHeaderAction from './page-header-action.vue'
import PageHeaderActionWithPrompt from './page-header-action-with-prompt.vue'
import type { PageHeaderAction as PageHeaderActionType } from './types'

defineProps<{
	actions: PageHeaderActionType[]
}>()
</script>
