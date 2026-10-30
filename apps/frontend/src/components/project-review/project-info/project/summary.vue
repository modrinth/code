<template>
	<Section heading="Summary" :target="{ kind: 'summary' }">
		<template #right>
			<EditButton v-if="!editing" section="Summary" :disabled="saving" @click="startEditing" />
		</template>
		<div v-if="editing" class="flex flex-col gap-1.5">
			<Textarea
				ref="summaryInput"
				v-model="draft"
				aria-label="Summary"
				:maxlength="256"
				:disabled="saving"
				resize="vertical"
			/>
			<p v-if="validationMessage" class="m-0 text-sm text-red" role="alert">
				{{ validationMessage }}
			</p>
			<div class="flex justify-end gap-1">
				<Button type="quiet" size="xs" :disabled="saving" @click="resetEditing"> Cancel </Button>
				<Button
					type="colored"
					color="brand"
					size="xs"
					:loading="saving"
					:disabled="saving || !valid"
					@click="save"
				>
					Save
				</Button>
			</div>
		</div>
		<p v-else class="m-0 whitespace-pre-wrap break-words leading-[1.5]">
			{{ project?.summary }}
		</p>
	</Section>
</template>

<script setup lang="ts">
import { Button, Textarea } from '@modrinth/ui'
import { useTemplateRef } from 'vue'

import EditButton from '../edit/button.vue'
import { useProjectTextEdit } from '../edit/use-project-text-edit'
import Section from '../section.vue'

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
