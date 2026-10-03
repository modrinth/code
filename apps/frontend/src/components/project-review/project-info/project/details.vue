<template>
	<Section :heading="formatMessage(messages.details)">
		<dl v-if="project" class="m-0 grid grid-cols-[5.5rem_minmax(0,1fr)] gap-x-3 gap-y-2">
			<dt class="font-medium">{{ formatMessage(messages.projectType) }}</dt>
			<dd class="m-0 flex flex-col gap-1">
				<div
					v-for="type in projectTypes"
					:key="type.id"
					class="flex items-center gap-1 font-semibold text-secondary"
				>
					<component :is="type.icon" class="size-4 shrink-0" aria-hidden="true" />
					<span
						:class="{
							underline: projectTypes.length > 1 && type.id === primaryProjectType,
						}"
						>{{ formatMessage(getProjectTypeTitleMessage(type.id), { count: 1 }) }}</span
					>
				</div>
			</dd>
			<dt class="font-medium">{{ formatMessage(messages.created) }}</dt>
			<dd class="m-0">
				<time :datetime="project.published" :title="project.published">{{
					relativeTime(project.published)
				}}</time>
			</dd>
			<dt class="font-medium">{{ formatMessage(messages.updated) }}</dt>
			<dd class="m-0">
				<time :datetime="project.updated" :title="project.updated">{{
					relativeTime(project.updated)
				}}</time>
			</dd>
			<template v-if="project.queued">
				<dt class="font-medium">{{ formatMessage(messages.submitted) }}</dt>
				<dd class="m-0 flex flex-wrap items-center gap-x-1.5">
					<time :datetime="project.queued" :title="project.queued">{{
						relativeTime(project.queued)
					}}</time>
					<template v-if="submissionCount">
						<BulletDivider aria-hidden="true" />
						<span>{{ formatMessage(messages.submissions, { count: submissionCount }) }}</span>
					</template>
				</dd>
			</template>
			<dt class="font-medium">{{ formatMessage(messages.downloads) }}</dt>
			<dd class="m-0">{{ formatNumber(project.downloads) }}</dd>
			<template v-if="project.requested_status">
				<dt class="font-medium">{{ formatMessage(messages.requesting) }}</dt>
				<dd class="m-0">
					<ProjectStatusBadge
						:status="project.requested_status"
						:style="{ color: `var(--color-${getProjectStatusColor(project.requested_status)})` }"
					/>
				</dd>
			</template>
		</dl>
	</Section>
</template>

<script setup lang="ts">
import { ServerIcon } from '@modrinth/assets'
import {
	BulletDivider,
	getProjectStatusColor,
	getProjectTypeIcon,
	getProjectTypeTitleMessage,
	ProjectStatusBadge,
	useFormatNumber,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import { getPrimaryProjectType } from '@modrinth/utils'
import { computed } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../../messages'
import Section from '../section.vue'

const { project, submissionCount } = injectProjectReviewPageContext()
const primaryProjectType = computed(() =>
	project.value ? getPrimaryProjectType(project.value) : undefined,
)
const projectTypes = computed(() => {
	if (project.value?.minecraft_server) return [{ id: 'server', icon: ServerIcon }]
	return (project.value?.project_types ?? [])
		.toSorted(
			(a, b) => Number(b === primaryProjectType.value) - Number(a === primaryProjectType.value),
		)
		.map((type) => ({ id: type, icon: getProjectTypeIcon(type) }))
})
const { formatMessage } = useVIntl()
const relativeTime = useRelativeTime({ style: 'narrow' })
const formatNumber = useFormatNumber()
</script>
