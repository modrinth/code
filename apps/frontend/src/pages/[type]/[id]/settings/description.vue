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
				v-model="current.description"
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
		</div>
		<UnsavedChangesPopup
			:original="saved"
			:modified="current"
			:saving="saving"
			@reset="reset"
			@save="save"
		/>
	</div>
</template>

<script lang="ts" setup>
import { TriangleAlertIcon } from '@modrinth/assets'
import { countText, MIN_DESCRIPTION_CHARS } from '@modrinth/moderation'
import {
	injectProjectPageContext,
	MarkdownEditor,
	UnsavedChangesPopup,
	useSavable,
} from '@modrinth/ui'
import { TeamMemberPermission } from '@modrinth/utils'
import { computed } from 'vue'

import { useImageUpload } from '~/composables/image-upload.ts'

const { projectV2: project, currentMember, patchProject } = injectProjectPageContext()

const { saved, current, saving, reset, save } = useSavable(
	() => ({ description: project.value.body }),
	async ({ description }) => {
		await patchProject({ body: description })
	},
)

const descriptionWarning = computed(() => {
	const text = current.value.description?.trim() || ''
	const charCount = countText(text)

	if (charCount < MIN_DESCRIPTION_CHARS) {
		return `It's recommended to have a description with at least ${MIN_DESCRIPTION_CHARS} readable characters. (${charCount}/${MIN_DESCRIPTION_CHARS})`
	}

	return null
})

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
