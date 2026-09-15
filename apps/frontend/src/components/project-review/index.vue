<template>
	<ClientOnly>
		<ProjectReviewWorkspace>
			<template v-for="(_, name) in slots" #[name]>
				<slot :name="name" />
			</template>
		</ProjectReviewWorkspace>
		<template #fallback>
			<p class="p-4 text-secondary" role="status">
				{{ formatMessage(projectReviewMessages.loading) }}
			</p>
		</template>
	</ClientOnly>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'

import { projectReviewMessages } from './messages'
import type { ProjectReviewSlots } from './types'
import ProjectReviewWorkspace from './workspace/workspace.client.vue'

const { formatMessage } = useVIntl()
const slots = defineSlots<ProjectReviewSlots>()
</script>
