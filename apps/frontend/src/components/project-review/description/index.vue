<template>
	<div
		ref="hotkeyScope"
		class="editable-review-section group/description flex h-full min-h-0 flex-col gap-2.5 overflow-y-auto overflow-x-hidden"
	>
		<div class="sticky top-0 z-10 flex shrink-0 items-start justify-between gap-2 bg-bg">
			<ReviewPanel
				mode="inline"
				:target="{ kind: 'description' }"
				:disabled="isLoading || !!error"
				:hotkey-scope="hotkeyScope"
				class="!w-auto min-w-0 flex-1 pb-2.5 group-hover/description:opacity-100"
			/>
			<div class="mt-0.5 flex shrink-0 justify-end gap-1">
				<template v-if="editing">
					<Button type="quiet" size="xs" :disabled="saving" @click="resetEditing">
						{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<Button
						type="colored"
						color="brand"
						size="xs"
						:loading="saving"
						:disabled="!valid"
						@click="save"
					>
						{{ formatMessage(commonMessages.saveButton) }}
					</Button>
				</template>
				<EditButton
					v-else-if="project && !isLoading && !error"
					:section="formatMessage(messages.description)"
					:disabled="saving"
					@click="startEditing"
				/>
			</div>
		</div>
		<div class="min-h-0 flex-1">
			<p v-if="!selection" class="m-0 text-secondary">
				{{ formatMessage(messages.empty) }}
			</p>
			<p v-else-if="isLoading" role="status" class="m-0">
				{{ formatMessage(messages.loading) }}
			</p>
			<div v-else-if="error" role="alert">
				<p class="m-0">{{ formatMessage(messages.loadError) }}</p>
				<Button @click="refresh">{{ formatMessage(messages.retry) }}</Button>
			</div>
			<div v-else-if="editing" class="flex min-h-full flex-col gap-2">
				<MarkdownEditor ref="editor" v-model="draft" :disabled="saving" />
				<p v-if="validationMessage" class="m-0 text-sm text-red" role="alert">
					{{ validationMessage }}
				</p>
			</div>
			<ProjectPageDescription
				v-else-if="project?.description?.trim()"
				class="text-sm leading-tight"
				:description="project.description"
			/>
			<p v-else class="m-0 text-secondary">
				{{ formatMessage(messages.emptyDescription) }}
			</p>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	Button,
	commonMessages,
	MarkdownEditor,
	ProjectPageDescription,
	useVIntl,
} from '@modrinth/ui'
import { useTemplateRef } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../messages'
import EditButton from '../project-info/edit/button.vue'
import { useProjectTextEdit } from '../project-info/edit/use-project-text-edit'
import ReviewPanel from '../review-panel/index.vue'

const { formatMessage } = useVIntl()
const hotkeyScope = useTemplateRef<HTMLElement>('hotkeyScope')
const { selection, isLoading, error, refresh } = injectProjectReviewPageContext()
const editor = useTemplateRef<InstanceType<typeof MarkdownEditor>>('editor')
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
} = useProjectTextEdit('description', () => editor.value?.focus())
</script>
