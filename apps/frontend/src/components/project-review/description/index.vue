<template>
	<div
		ref="interactionScope"
		class="editable-review-section flex h-full min-h-0 flex-col gap-2.5 overflow-y-auto overflow-x-hidden px-2 pb-2"
	>
		<div
			class="sticky top-0 z-10 flex shrink-0 items-start justify-between gap-4 bg-surface-1 pt-2"
		>
			<ReviewPanel
				mode="inline"
				:target="{ kind: 'description' }"
				hide-title
				:disabled="isLoading || !!error"
				:interaction-scope="interactionScope"
				class="!w-auto min-w-0 flex-1 pb-2.5"
			/>
			<div class="mt-0.5 flex shrink-0 justify-end gap-1">
				<template v-if="editing">
					<Button type="quiet" size="xs" :disabled="saving" @click="resetEditing"> Cancel </Button>
					<Button
						type="colored"
						color="brand"
						size="xs"
						:loading="saving"
						:disabled="!valid"
						@click="save"
					>
						Save
					</Button>
				</template>
				<EditButton
					v-else-if="project && !isLoading && !error"
					section="Description"
					:disabled="saving"
					@click="startEditing"
				/>
			</div>
		</div>
		<div class="min-h-0 flex-1">
			<p v-if="!selection" class="m-0 text-secondary">No project selected</p>
			<p v-else-if="isLoading" role="status" class="m-0">Loading...</p>
			<div v-else-if="error" role="alert">
				<p class="m-0">Could not load project data</p>
				<Button @click="refresh">Retry</Button>
			</div>
			<div v-else-if="editing" class="flex min-h-full flex-col gap-2">
				<TemplateMarkdownEditor
					ref="editor"
					v-model="draft"
					:disabled="saving"
					:hide-formatting-buttons="
						settings.get(moderationSettings.General.HideMarkdownFormattingButtons)
					"
				/>
				<p v-if="validationMessage" class="m-0 text-sm text-red" role="alert">
					{{ validationMessage }}
				</p>
			</div>
			<ProjectPageDescription
				v-else-if="project?.description?.trim()"
				class="text-sm leading-tight"
				:description="project.description"
			/>
			<p v-else class="m-0 text-secondary">No description</p>
		</div>
	</div>
</template>

<script setup lang="ts">
import { moderationSettings } from '@modrinth/moderation'
import { Button, ProjectPageDescription } from '@modrinth/ui'
import { useTemplateRef } from 'vue'

import TemplateMarkdownEditor from '~/components/project-review/message-thread/template-markdown-editor.vue'
import { useModerationSettings } from '~/composables/moderation'
import { injectProjectReviewPageContext } from '~/providers/project-review'

import EditButton from '../project-info/edit/button.vue'
import { useProjectTextEdit } from '../project-info/edit/use-project-text-edit'
import ReviewPanel from '../review-panel/index.vue'

const settings = useModerationSettings()
const interactionScope = useTemplateRef<HTMLElement>('interactionScope')
const { selection, isLoading, error, refresh } = injectProjectReviewPageContext()
const editor = useTemplateRef<InstanceType<typeof TemplateMarkdownEditor>>('editor')
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
