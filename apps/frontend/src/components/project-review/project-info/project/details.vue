<template>
	<Section :heading="formatMessage(messages.details)">
		<template #right>
			<EditButton :section="formatMessage(messages.details)" @click="editModal?.show()" />
		</template>
		<EditModal :key="project?.id" ref="editModal" />
		<dl v-if="project" class="m-0 grid grid-cols-[5.5rem_minmax(0,1fr)] gap-x-3 gap-y-2">
			<dt class="font-medium">{{ formatMessage(messages.project) }}</dt>
			<dd class="m-0 flex min-w-0 items-center gap-1">
				<Tooltip class="flex shrink-0">
					<template #popper>
						<ProjectStatusBadge
							:status="project.status"
							:style="{ color: `var(--color-${getProjectStatusColor(project.status)})` }"
						/>
					</template>
					<component
						:is="getProjectStatusIcon(project.status)"
						class="size-4 shrink-0"
						:style="{ color: `var(--color-${getProjectStatusColor(project.status)})` }"
						aria-hidden="true"
					/>
				</Tooltip>
				<NuxtLink
					:to="projectUrl"
					target="_blank"
					rel="noopener noreferrer"
					class="min-w-0 break-words font-semibold text-secondary hover:underline"
				>
					{{ project.name }}
				</NuxtLink>
			</dd>
			<template v-if="project.requested_status">
				<dt class="font-medium">{{ formatMessage(messages.requesting) }}</dt>
				<dd class="m-0">
					<ProjectStatusBadge
						:status="project.requested_status"
						:style="{ color: `var(--color-${getProjectStatusColor(project.requested_status)})` }"
					/>
				</dd>
			</template>
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
			<dt class="font-medium">{{ formatMessage(messages.monetization) }}</dt>
			<dd class="m-0">
				{{ formatMessage(messages.monetizationValue, { status: monetizationStatus }) }}
			</dd>

			<dt class="self-center font-medium">{{ formatMessage(messages.actions) }}</dt>
			<dd class="m-0 flex items-center gap-2">
				<CopyCode :text="project.id" :display-text="formatMessage(messages.projectId)" />
				<CopyCode
					:text="`${config.public.siteUrl}/project/${project.id}`"
					:display-text="formatMessage(messages.permalink)"
				/>
			</dd>
		</dl>
	</Section>
</template>

<script setup lang="ts">
import { ServerIcon } from '@modrinth/assets'
import {
	BulletDivider,
	CopyCode,
	getProjectStatusColor,
	getProjectStatusIcon,
	getProjectTypeIcon,
	getProjectTypeTitleMessage,
	ProjectStatusBadge,
	Tooltip,
	useFormatNumber,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import { getPrimaryProjectType } from '@modrinth/utils'
import { computed, useTemplateRef } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../../messages'
import EditButton from '../edit/button.vue'
import EditModal from '../edit/details.vue'
import Section from '../section.vue'

const { project, projectV2, submissionCount } = injectProjectReviewPageContext()
const config = useRuntimeConfig()
const editModal = useTemplateRef<InstanceType<typeof EditModal>>('editModal')
const monetizationStatus = computed(() =>
	project.value?.monetization_status === 'force-demonetized'
		? 'forceDemonetized'
		: project.value?.monetization_status,
)
const projectUrl = computed(
	() =>
		`/${projectV2.value?.project_type ?? project.value?.project_types[0]}/${project.value?.slug ?? project.value?.id}`,
)
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
