<template>
	<Section :heading="formatMessage(messages.slug)" :target="{ kind: 'slug' }">
		<template #actions>
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
			<div class="break-all text-sm">
				<span>/{{ projectTypeForUrl }}/</span>
				<span class="inline-flex text-contrast">{{ project.slug ?? project.id }}</span>
			</div>
		</div>
	</Section>
</template>

<script setup lang="ts">
import { Button, commonMessages, Input, useVIntl } from '@modrinth/ui'
import { computed, useTemplateRef } from 'vue'

import { getProjectTypeForUrl } from '~/helpers/projects.js'

import { projectReviewMessages as messages } from '../../messages'
import EditButton from '../edit/button.vue'
import { useProjectTextEdit } from '../edit/use-project-text-edit'
import Section from '../section.vue'

const { formatMessage } = useVIntl()
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

const projectTypeForUrl = computed(() => {
	if (project.value?.minecraft_server != null) return 'server'
	const type = project.value?.project_types[0] ?? 'mod'
	return getProjectTypeForUrl(type, project.value?.loaders ?? [])
})
</script>
