<template>
	<div>
		<div class="universal-card">
			<div class="markdown-disclaimer">
				<h2>Description</h2>
				<span class="label__description">
					You can type an extended description of your project here.
					<span class="label__subdescription">
						The description must clearly and honestly describe the purpose and function of the
						project. See section 2.1 of the
						<nuxt-link class="text-link" target="_blank" to="/legal/rules">Content Rules</nuxt-link>
						for the full requirements.
					</span>
				</span>
			</div>
			<MarkdownEditor
				v-model="description"
				:disabled="
					!currentMember ||
					(currentMember?.permissions! & TeamMemberPermission.EDIT_BODY) !==
						TeamMemberPermission.EDIT_BODY
				"
				:on-image-upload="onUploadHandler"
			/>
			<div v-if="descriptionWarning" class="flex items-center gap-1.5 text-orange">
				<TriangleAlertIcon class="my-auto" />
				{{ descriptionWarning }}
			</div>
			<div class="input-group markdown-disclaimer">
				<button
					:disabled="!hasChanges"
					class="iconified-button brand-button"
					type="button"
					@click="saveChanges()"
				>
					<SaveIcon />
					Save changes
				</button>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
import { SaveIcon, TriangleAlertIcon } from '@modrinth/assets'
import { countText, MIN_DESCRIPTION_CHARS } from '@modrinth/moderation'
import { injectProjectPageContext, MarkdownEditor } from '@modrinth/ui'
import { TeamMemberPermission } from '@modrinth/utils'
import { computed, ref } from 'vue'

import { useImageUpload } from '~/composables/image-upload.ts'

const { projectV2: project, currentMember, patchProject } = injectProjectPageContext()

const description = ref(project.value.body)

const descriptionWarning = computed(() => {
	const text = description.value?.trim() || ''
	const charCount = countText(text)

	if (charCount < MIN_DESCRIPTION_CHARS) {
		return `It's recommended to have a description with at least ${MIN_DESCRIPTION_CHARS} readable characters. (${charCount}/${MIN_DESCRIPTION_CHARS})`
	}

	return null
})

const patchRequestPayload = computed(() => {
	const payload: {
		body?: string
	} = {}

	if (description.value !== project.value.body) {
		payload.body = description.value
	}

	return payload
})

const hasChanges = computed(() => {
	return Object.keys(patchRequestPayload.value).length > 0
})

function saveChanges() {
	patchProject(patchRequestPayload.value)
}

async function onUploadHandler(file: File) {
	const response = await useImageUpload(file, {
		context: 'project',
		projectID: project.value.id,
	})

	return response.url
}
</script>

<style scoped>
.markdown-disclaimer {
	margin-block: 1rem;
}
</style>
