<template>
	<div>
		<AiImageWarningModal ref="aiImageWarningModal" />
		<ConfirmLeaveModal ref="confirmLeaveModal" />
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
				:disabled="!hasPermission"
				:on-image-upload="onUploadHandler"
			/>
			<ValidationMessage :check="descriptionValidation" class="mt-2" />
		</div>
		<UnsavedChangesPopup
			:original="saved"
			:modified="current"
			:saving="saving"
			:can-save="canSave"
			@reset="reset"
			@save="save"
		/>
	</div>
</template>

<script lang="ts" setup>
import {
	commonProjectSettingsMessages,
	ConfirmLeaveModal,
	injectProjectPageContext,
	MarkdownEditor,
	UnsavedChangesPopup,
	usePageLeaveSafety,
	useSavable,
} from '@modrinth/ui'
import { isAdmin, TeamMemberPermission } from '@modrinth/utils'
import { computed, useTemplateRef } from 'vue'

import AiImageWarningModal from '~/components/ui/AiImageWarningModal.vue'
import ValidationMessage from '~/components/ValidationMessage.vue'
import { useImageUpload } from '~/composables/image-upload.ts'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { fileDeclaresAi } from '~/helpers/c2pa'

const { projectV2: project, currentMember, patchProject } = injectProjectPageContext()
const aiImageWarningModal = useTemplateRef('aiImageWarningModal')

useProjectSettingsHeadTitle(commonProjectSettingsMessages.description)

const {
	saved,
	current,
	saving,
	hasChanges,
	reset,
	save: saveForm,
} = useSavable(
	() => ({ description: project.value.body }),
	async ({ description }) => {
		await patchProject({ body: description })
	},
)

const { confirmLeaveModal } = usePageLeaveSafety(hasChanges)

const isAdminUser = computed(() => isAdmin(currentMember.value?.user))
const hasPermission = computed(
	() =>
		isAdminUser.value ||
		(!!currentMember.value &&
			(currentMember.value.permissions & TeamMemberPermission.EDIT_BODY) ===
				TeamMemberPermission.EDIT_BODY),
)
const descriptionValidation = useProjectNagMessages('description')
const canSave = computed(() => hasPermission.value)

async function save() {
	if (!canSave.value) return
	await saveForm()
}

async function onUploadHandler(file: File) {
	if (await fileDeclaresAi(file)) {
		aiImageWarningModal.value?.show()
		return
	}
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
