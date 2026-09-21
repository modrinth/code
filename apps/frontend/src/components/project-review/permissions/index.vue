<template>
	<section class="flex flex-col gap-3">
		<h2 class="m-0 text-lg font-semibold text-contrast">
			{{ formatMessage(messages.permissions) }}
		</h2>
		<ReviewPanel
			v-if="panels.resolve({ kind: 'permissions' })"
			mode="inline"
			:target="{ kind: 'permissions' }"
		/>
		<p v-else-if="permissions.loading" class="m-0 text-secondary">
			{{ formatMessage(messages.loading) }}
		</p>
		<p v-else-if="permissions.error" class="m-0 text-red" role="alert">
			{{ formatMessage(messages.loadError) }}
		</p>
		<p v-else class="m-0 text-secondary">{{ formatMessage(messages.noReviewActions) }}</p>
	</section>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'

import { injectProjectReviewPageContext } from '~/providers/project-review'
import { injectReviewPanels } from '~/providers/project-review/review-panels'

import { projectReviewMessages as messages } from '../messages'
import ReviewPanel from '../review-panel/index.vue'

const panels = injectReviewPanels()
const { permissions } = injectProjectReviewPageContext()
const { formatMessage } = useVIntl()
</script>
