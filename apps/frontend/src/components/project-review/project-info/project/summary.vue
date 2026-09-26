<template>
	<Section :heading="formatMessage(messages.summary)" :target="{ kind: 'summary' }">
		<template #actions>
			<EditButton
				v-if="!editing"
				:section="formatMessage(messages.summary)"
				:disabled="saving"
				@click="startEditing"
			/>
		</template>
		<div v-if="editing" class="flex flex-col gap-1.5">
			<Textarea
				ref="summaryInput"
				v-model="draft"
				:aria-label="formatMessage(messages.summary)"
				:maxlength="256"
				:disabled="saving"
				resize="vertical"
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
		<p v-else class="m-0 whitespace-pre-wrap break-words leading-[1.5]">
			{{ project?.summary }}
		</p>
	</Section>
</template>

<script setup lang="ts">
import { Button, commonMessages, Textarea, useVIntl } from '@modrinth/ui'
import { useTemplateRef } from 'vue'

import { projectReviewMessages as messages } from '../../messages'
import EditButton from '../edit/button.vue'
import { useProjectTextEdit } from '../edit/use-project-text-edit'
import Section from '../section.vue'

const { formatMessage } = useVIntl()
const summaryInput = useTemplateRef<InstanceType<typeof Textarea>>('summaryInput')
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
} = useProjectTextEdit('summary', () => summaryInput.value?.focus())
</script>
