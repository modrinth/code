<template>
	<dl v-if="project" class="m-0 grid grid-cols-[5.5rem_minmax(0,1fr)] gap-x-3 gap-y-2">
		<dt>{{ formatMessage(messages.projectType) }}</dt>
		<dd class="m-0 capitalize">
			<template v-for="(type, index) in projectTypes" :key="type">
				<template v-if="index > 0">, </template>
				<span
					:class="{
						underline: project.project_types.length > 1 && type === primaryProjectType,
					}"
					>{{ type }}</span
				>
			</template>
		</dd>
		<dt>{{ formatMessage(messages.created) }}</dt>
		<dd class="m-0">
			<time :datetime="project.published" :title="project.published">{{
				relativeTime(project.published)
			}}</time>
		</dd>
		<dt>{{ formatMessage(messages.updated) }}</dt>
		<dd class="m-0">
			<time :datetime="project.updated" :title="project.updated">{{
				relativeTime(project.updated)
			}}</time>
		</dd>
		<template v-if="project.queued">
			<dt>{{ formatMessage(messages.submitted) }}</dt>
			<dd class="m-0">
				<time :datetime="project.queued" :title="project.queued">{{
					relativeTime(project.queued)
				}}</time
				><template v-if="submissionCount">
					·
					{{ formatMessage(messages.submissions, { count: submissionCount }) }}</template
				>
			</dd>
		</template>
		<dt>{{ formatMessage(messages.downloads) }}</dt>
		<dd class="m-0">{{ formatNumber(project.downloads) }}</dd>
		<template v-if="project.requested_status"
			><dt>{{ formatMessage(messages.requesting) }}</dt>
			<dd class="m-0 capitalize">{{ project.requested_status }}</dd></template
		>
	</dl>
</template>

<script setup lang="ts">
import { useFormatNumber, useRelativeTime, useVIntl } from '@modrinth/ui'
import { getPrimaryProjectType } from '@modrinth/utils'
import { computed } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../../messages'

const { project, submissionCount } = injectProjectReviewPageContext()
const primaryProjectType = computed(() =>
	project.value ? getPrimaryProjectType(project.value) : undefined,
)
const projectTypes = computed(() =>
	(project.value?.project_types ?? []).toSorted(
		(a, b) => Number(b === primaryProjectType.value) - Number(a === primaryProjectType.value),
	),
)
const { formatMessage } = useVIntl()
const relativeTime = useRelativeTime({ style: 'narrow' })
const formatNumber = useFormatNumber()
</script>
