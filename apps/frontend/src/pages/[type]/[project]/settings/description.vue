<template>
	<div>
		<AiImageWarningModal ref="aiImageWarningModal" />
		<ConfirmLeaveModal ref="confirmLeaveModal" />
		<h2 class="settings-section-title">{{ formatMessage(messages.title) }}</h2>
		<p class="mb-4 mt-0 text-secondary">{{ formatMessage(messages.subtitle) }}</p>
		<div>
			<div class="mb-4 flex flex-col gap-2.5 empty:hidden">
				<ProjectIssueCard target="modify_description" :field-action="descriptionIssueAction" />
				<ValidationMessage
					:check="descriptionValidation"
					:project-field="saved.description"
					:current-field="current.description"
				/>
				<ValidationMessage :check="saveValidation.forField('description')" />
			</div>

			<MarkdownEditor
				v-model="current.description"
				:placeholder="formatMessage(messages.placeholder)"
				:disabled="saving || !hasPermission"
				:on-image-upload="onUploadHandler"
				integrated
				:min-height="480"
			>
				<template #guidance>
					<Accordion
						class="border-0 border-b border-solid border-surface-5"
						button-class="m-0 flex w-full cursor-pointer items-center gap-2 border-0 bg-transparent px-5 py-3 text-sm text-secondary"
						content-class="px-5 pb-4 text-sm leading-relaxed text-secondary"
					>
						<template #button="{ open }">
							<InfoIcon class="size-4 shrink-0" />
							{{ formatMessage(messages.guidanceTitle) }}
							<ChevronDownIcon
								class="ml-auto size-4 transition-transform motion-reduce:transition-none"
								:class="{ 'rotate-180': open }"
							/>
						</template>
						<p class="m-0">{{ formatMessage(messages.guidanceIntro) }}</p>
						<ul class="my-3 space-y-2 pl-5">
							<li>{{ formatMessage(messages.guidanceAi) }}</li>
							<li>
								<IntlFormatted :message-id="messages.guidanceTranslation">
									<template #deepl="{ children }">
										<a
											class="font-semibold text-link"
											href="https://www.deepl.com/translator"
											target="_blank"
											rel="noopener noreferrer"
											><component :is="() => children"
										/></a>
									</template>
								</IntlFormatted>
							</li>
						</ul>
						<p class="m-0">
							<IntlFormatted :message-id="messages.guidanceRules">
								<template #rules="{ children }">
									<NuxtLink class="font-semibold text-link" target="_blank" to="/legal/rules"
										><component :is="() => children"
									/></NuxtLink>
								</template>
							</IntlFormatted>
						</p>
					</Accordion>
				</template>
			</MarkdownEditor>
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
import { DropdownIcon as ChevronDownIcon, InfoIcon } from '@modrinth/assets'
import {
	Accordion,
	commonProjectSettingsMessages,
	ConfirmLeaveModal,
	defineMessages,
	injectNotificationManager,
	injectProjectPageContext,
	IntlFormatted,
	MarkdownEditor,
	UnsavedChangesPopup,
	usePageLeaveSafety,
	useSavable,
	useVIntl,
} from '@modrinth/ui'
import { isAdmin, TeamMemberPermission } from '@modrinth/utils'
import { computed, useTemplateRef } from 'vue'

import AiImageWarningModal from '~/components/ui/AiImageWarningModal.vue'
import ProjectIssueCard from '~/components/ui/project-issue-card/index.vue'
import ValidationMessage from '~/components/ValidationMessage.vue'
import { useImageUpload } from '~/composables/image-upload.ts'
import { useProjectIssueFieldAction } from '~/composables/project-issue-field-action'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { useProjectSaveValidation } from '~/composables/project-save-validation'
import { fileDeclaresAi } from '~/helpers/c2pa'

const { projectV2: project, currentMember, patchProjectV3 } = injectProjectPageContext()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	placeholder: {
		id: 'project.settings.description.placeholder',
		defaultMessage: 'Describe your project...',
	},
	subtitle: {
		id: 'project.settings.description.subtitle',
		defaultMessage: 'Explain what your project does, how to install it, and what it works with.',
	},
	guidanceTitle: {
		id: 'project.settings.description.guidance-title',
		defaultMessage: 'What makes a good description?',
	},
	title: { id: 'project.settings.description.title', defaultMessage: 'Description' },
	guidanceIntro: {
		id: 'project.settings.description.guidance-intro',
		defaultMessage:
			'Project descriptions must make a clear and honest attempt at describing what your project does so that anyone who comes across your project can understand what it adds, changes, or removes before downloading it.',
	},
	guidanceAi: {
		id: 'project.settings.description.guidance-ai',
		defaultMessage:
			'Do not use AI to generate a description. Descriptions must be accurate, and AI tends to make up things that aren’t true about your project.',
	},
	guidanceTranslation: {
		id: 'project.settings.description.guidance-translation',
		defaultMessage:
			'If your description is in a language other than English, an English translation must also be available unless the project is only meant to be used in a particular language, such as a translation pack. Use of machine translation such as <deepl>DeepL</deepl> is allowed.',
	},
	guidanceRules: {
		id: 'project.settings.description.guidance-rules',
		defaultMessage:
			'Please refer to Section 2.1 of Modrinth’s <rules>Content Rules</rules> for more information.',
	},
	updated: { id: 'project.settings.description.updated', defaultMessage: 'Description updated' },
	updatedText: {
		id: 'project.settings.description.updated-text',
		defaultMessage: 'Your description has been updated.',
	},
})
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
		await patchProjectV3({ description }, true, true)
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
const descriptionValidation = useProjectNagMessages('description', 'description')
const saveValidation = useProjectSaveValidation(() => current.value)
const canSave = computed(
	() =>
		hasPermission.value &&
		!saveValidation.messages.value.some((message) => message.severity === 'error'),
)

const descriptionIssueAction = useProjectIssueFieldAction({
	draft: () => ({ description: current.value.description }),
	canSave: () => canSave.value,
	saving,
	validation: saveValidation,
	save: async () => {
		const description = current.value.description
		if (description !== saved.value.description) {
			await patchProjectV3({ description }, true, true)
		}
	},
})

async function save() {
	if (!canSave.value || saving.value) return
	const submittedState = saveValidation.snapshot()
	try {
		await saveForm()
		saveValidation.clear()
		addNotification({
			title: formatMessage(messages.updated),
			text: formatMessage(messages.updatedText),
			type: 'success',
		})
	} catch (error) {
		saveValidation.capture(error, submittedState)
	}
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
