<template>
	<Section :heading="formatMessage(messages.slug)" :target="{ kind: 'slug' }">
		<template #right>
			<EditButton
				v-if="!editing"
				:section="formatMessage(messages.slug)"
				:disabled="saving"
				@click="startEditing"
			/>
		</template>
		<div v-if="editing" class="flex flex-col gap-1.5">
			<Input
				ref="input"
				v-model="draft"
				:aria-label="formatMessage(messages.slug)"
				:maxlength="64"
				autocomplete="off"
				:disabled="saving"
				wrapper-class="w-full"
			>
				<template #prefix>/{{ projectTypeForUrl }}/</template>
			</Input>
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
		<div v-else-if="project" class="min-w-0">
			<div class="flex items-center gap-1.5 text-sm">
				<span class="min-w-0 break-all">
					<span>/{{ projectTypeForUrl }}/</span>
					<span class="inline-flex text-contrast">{{ project.slug ?? project.id }}</span>
				</span>
				<TriangleAlertIcon
					v-if="isNonSuggestedSlug"
					v-tooltip="formatMessage(messages.slugNotSuggested)"
					:aria-label="formatMessage(messages.slugNotSuggested)"
					class="size-4 shrink-0 text-orange"
				/>
			</div>
		</div>
	</Section>
</template>

<script setup lang="ts">
import { TriangleAlertIcon } from '@modrinth/assets'
import { Button, commonMessages, Input, useVIntl } from '@modrinth/ui'
import { computed, useTemplateRef } from 'vue'

import { generateProjectSlugSuggestions } from '~/composables/project-slug-suggestions'
import { getProjectTypeForUrl } from '~/helpers/projects.js'
import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../../messages'
import EditButton from '../edit/button.vue'
import { useProjectTextEdit } from '../edit/use-project-text-edit'
import Section from '../section.vue'

const { formatMessage } = useVIntl()
const { members } = injectProjectReviewPageContext()
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
} = useProjectTextEdit('slug', () => input.value?.focus())

const isNonSuggestedSlug = computed(() => {
	if (!project.value) return false
	const owner = members.value.find((member) => member.is_owner) ?? members.value[0]
	const suggestions = generateProjectSlugSuggestions(project.value.name, owner?.user.username)
	return !suggestions.includes(project.value.slug?.toLowerCase() ?? '')
})

const projectTypeForUrl = computed(() => {
	if (project.value?.minecraft_server != null) return 'server'
	const type = project.value?.project_types[0] ?? 'mod'
	return getProjectTypeForUrl(type, project.value?.loaders ?? [])
})
</script>
