<template>
	<Tooltip
		theme="dismissable-prompt"
		:triggers="[]"
		:shown="action.prompt?.shown"
		:auto-hide="false"
		:placement="action.prompt?.placement ?? 'bottom'"
	>
		<PageHeaderAction :action="action" @dismiss-prompt="dismissPrompt" />
		<template v-if="action.prompt" #popper>
			<PageHeaderActionPromptPopper :prompt="action.prompt" />
		</template>
	</Tooltip>
</template>

<script setup lang="ts">
import { Tooltip } from 'floating-vue'

import PageHeaderAction from './page-header-action.vue'
import PageHeaderActionPromptPopper from './page-header-action-prompt-popper.vue'
import type { PageHeaderAction as PageHeaderActionType } from './types'

const props = defineProps<{
	action: PageHeaderActionType
}>()

function dismissPrompt() {
	if (props.action.prompt?.shown) {
		props.action.prompt.onDismiss?.()
	}
}
</script>
