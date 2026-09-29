<template>
	<Section :heading="formatMessage(messages.projectTitle)" :target="{ kind: 'title' }">
		<template #right>
			<EditButton
				v-if="!editing"
				:section="formatMessage(messages.projectTitle)"
				:disabled="saving"
				@click="startEditing"
			/>
		</template>
		<div v-if="editing" class="flex flex-col gap-1.5">
			<Input
				ref="input"
				v-model="draft"
				:aria-label="formatMessage(messages.projectTitle)"
				:maxlength="64"
				:disabled="saving"
				wrapper-class="w-full"
			/>
			<p v-if="validationMessage" class="m-0 text-sm text-red" role="alert">
				{{ validationMessage }}
			</p>
			<div class="flex justify-end gap-1">
				<Button type="quiet" size="xs" :disabled="saving" @click="resetEditing">
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="brand"
					size="xs"
					:loading="saving"
					:disabled="saving || !valid"
					@click="save"
				>
					{{ formatMessage(commonMessages.saveButton) }}
				</Button>
			</div>
		</div>
		<h1 v-else-if="project" class="m-0 flex min-w-0 items-center gap-2 text-xl">
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
				class="min-w-0 break-words text-contrast hover:underline"
			>
				{{ project.name }}
			</NuxtLink>
		</h1>
	</Section>
</template>

<script setup lang="ts">
import {
	Button,
	commonMessages,
	getProjectStatusColor,
	getProjectStatusIcon,
	Input,
	ProjectStatusBadge,
	Tooltip,
	useVIntl,
} from '@modrinth/ui'
import { computed, useTemplateRef } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../../messages'
import EditButton from '../edit/button.vue'
import { useProjectTextEdit } from '../edit/use-project-text-edit'
import Section from '../section.vue'

const { formatMessage } = useVIntl()
const { projectV2 } = injectProjectReviewPageContext()
const input = useTemplateRef<InstanceType<typeof Input>>('input')
const {
	project,
	saving,
	editing,
	draft,
	valid,
	validationMessage,
	startEditing,
	resetEditing,
	save,
} = useProjectTextEdit('name', () => input.value?.focus())

const projectUrl = computed(
	() =>
		`/${projectV2.value?.project_type ?? project.value?.project_types[0]}/${project.value?.slug ?? project.value?.id}`,
)
</script>
