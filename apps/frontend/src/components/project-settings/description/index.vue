<template>
	<div>
		<AiImageWarningModal ref="aiImageWarningModal" />
		<ConfirmLeaveModal ref="confirmLeaveModal" />
		<SettingsSection :title="formatMessage(messages.title)" class="mb-3" />
		<div
			class="[&_.cm-editor]:text-sm"
			data-project-field="description"
			data-project-field-scroll-only
		>
			<div class="mb-4 flex flex-col gap-2.5 empty:hidden">
				<ProjectIssueCard target="modify_description" :field-action="descriptionIssueAction" />
				<ValidationMessage
					:check="descriptionValidation"
					:project-field="saved.description"
					:current-field="current.description"
				/>
				<ValidationMessage :check="saveValidation.forField('description')" />
			</div>

			<Accordion
				v-if="mounted"
				class="mb-4 min-w-0 rounded-2xl border border-solid border-surface-4 bg-surface-3 px-4 pb-1"
				button-class="m-0 flex w-full cursor-pointer items-center gap-2 border-0 bg-transparent px-0 pb-3 pt-4 text-left text-base font-semibold text-contrast"
				content-class="pb-3 text-sm leading-relaxed text-primary"
				:open-by-default="guidanceOpen"
				@on-open="guidanceOpen = true"
				@on-close="guidanceOpen = false"
			>
				<template #button="{ open }">
					<InfoIcon class="size-5 shrink-0 text-secondary" aria-hidden="true" />
					<span class="min-w-0 flex-1 [overflow-wrap:anywhere]">
						{{ formatMessage(messages.guidanceTitle) }}
					</span>
					<ChevronDownIcon
						class="mr-1.5 size-5 shrink-0 text-secondary transition-transform duration-300 motion-reduce:transition-none"
						:class="{ 'rotate-180': open }"
						aria-hidden="true"
					/>
				</template>
				<p class="m-0">
					{{
						formatMessage(isServerProject ? messages.serverGuidanceIntro : messages.guidanceIntro, {
							contentType,
						})
					}}
				</p>
				<p class="mb-0 mt-3">{{ formatMessage(messages.guidelinesHeading) }}</p>
				<ul class="my-3 list-none space-y-2 p-0" role="list">
					<li v-for="guideline in guidelines" :key="guideline.id" class="flex items-start gap-2">
						<CheckIcon class="mt-1 size-4 shrink-0 text-green" aria-hidden="true" />
						<span>{{ formatMessage(guideline, { contentType }) }}</span>
					</li>
				</ul>
				<p class="m-0">{{ formatMessage(messages.mistakesHeading) }}</p>
				<ul class="my-3 list-none space-y-2 p-0" role="list">
					<li v-for="mistake in mistakes" :key="mistake.id" class="flex items-start gap-2">
						<XCircleIcon class="mt-1 size-4 shrink-0 text-red" aria-hidden="true" />
						<span>{{ formatMessage(mistake, { contentType }) }}</span>
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

			<DescriptionMarkdownEditor
				v-model="current.description"
				:placeholder="formatMessage(messages.placeholder)"
				:disabled="saving || !hasPermission"
				:on-image-upload="onUploadHandler"
			/>
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
import { CheckIcon, DropdownIcon as ChevronDownIcon, InfoIcon, XCircleIcon } from '@modrinth/assets'
import {
	Accordion,
	ConfirmLeaveModal,
	defineMessages,
	injectNotificationManager,
	injectProjectPageContext,
	IntlFormatted,
	UnsavedChangesPopup,
	usePageLeaveSafety,
	useSavable,
	useVIntl,
} from '@modrinth/ui'
import { isAdmin, TeamMemberPermission } from '@modrinth/utils'
import { useLocalStorage, useMounted } from '@vueuse/core'
import { computed, useTemplateRef } from 'vue'

import AiImageWarningModal from '~/components/ui/AiImageWarningModal.vue'
import ProjectIssueCard from '~/components/ui/project-issue-card/index.vue'
import ValidationMessage from '~/components/ValidationMessage.vue'
import { useImageUpload } from '~/composables/image-upload.ts'
import { useProjectIssueFieldAction } from '~/composables/project-issue-field-action'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { useProjectSaveValidation } from '~/composables/project-save-validation'
import { fileDeclaresAi } from '~/helpers/c2pa'
import { projectCopyType } from '~/utils/project-messages'

import SettingsSection from '../settings-section.vue'
import DescriptionMarkdownEditor from './description-markdown-editor.vue'

const { projectV2: project, projectV3, currentMember, patchProjectV3 } = injectProjectPageContext()
const isServerProject = computed(() => projectV3.value.minecraft_server != null)
const contentType = computed(() => projectCopyType(projectV3.value))
const guidanceOpen = useLocalStorage('project-description-guidance-open', true, {
	initOnMounted: true,
})
const mounted = useMounted()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	placeholder: {
		id: 'project.settings.description.placeholder',
		defaultMessage: 'Describe your project...',
	},
	guidanceTitle: {
		id: 'project.settings.description.guidance-title',
		defaultMessage: 'What makes a good description?',
	},
	title: { id: 'project.settings.description.title', defaultMessage: 'Description' },
	guidanceIntro: {
		id: 'project.settings.description.guidance.introduction',
		defaultMessage:
			'The description is the most important part of your project page and will be the deciding factor for someone choosing to download your content! A good description must explain the content, purpose, and appeal of your {contentType, select, mod {mod} modpack {modpack} resourcepack {resource pack} shader {shader} plugin {plugin} datapack {data pack} server {server} other {project}} to new users.',
	},
	serverGuidanceIntro: {
		id: 'project.settings.description.guidance.server-introduction',
		defaultMessage:
			'The description is the most important part of your server page and will be the deciding factor for someone choosing to join your server! A good description must explain the content, purpose, and appeal of your server to new players.',
	},
	guidelinesHeading: {
		id: 'project.settings.description.guidance.guidelines-heading',
		defaultMessage: 'Great descriptions follow a few key guidelines:',
	},
	explainAppeal: {
		id: 'project.settings.description.guidance.explain-appeal',
		defaultMessage:
			'Explain what makes your {contentType, select, mod {mod} modpack {modpack} resourcepack {resource pack} shader {shader} plugin {plugin} datapack {data pack} server {server} other {project}} interesting.',
	},
	explainUse: {
		id: 'project.settings.description.guidance.explain-use',
		defaultMessage:
			'Explain how your {contentType, select, mod {mod} modpack {modpack} resourcepack {resource pack} shader {shader} plugin {plugin} datapack {data pack} server {server} other {project}} can be used or played with.',
	},
	showGallery: {
		id: 'project.settings.description.guidance.show-gallery',
		defaultMessage: 'Showcase images from the Gallery, if applicable.',
	},
	accessibleFormatting: {
		id: 'project.settings.description.guidance.accessible-formatting',
		defaultMessage: 'Make proper use of formatting and be accessible to all users.',
	},
	mistakesHeading: {
		id: 'project.settings.description.guidance.mistakes-heading',
		defaultMessage: 'Avoid these common mistakes:',
	},
	avoidAssumptions: {
		id: 'project.settings.description.guidance.avoid-assumptions',
		defaultMessage:
			'Do NOT assume users already understand your {contentType, select, mod {mod} modpack {modpack} resourcepack {resource pack} shader {shader} plugin {plugin} datapack {data pack} server {server} other {project}} or its functionality.',
	},
	avoidServerAssumptions: {
		id: 'project.settings.description.guidance.avoid-server-assumptions',
		defaultMessage: 'Do NOT assume players already understand your server or its functionality.',
	},
	avoidMisleadingClaims: {
		id: 'project.settings.description.guidance.avoid-misleading-claims',
		defaultMessage:
			'Do NOT make hyperbolic or misleading claims, or otherwise attempt to deceive users.',
	},
	avoidRepetition: {
		id: 'project.settings.description.guidance.avoid-repetition',
		defaultMessage:
			'Do NOT add repetitive or unnecessary information, such as installation instructions, unless unique to the {contentType, select, mod {mod} modpack {modpack} resourcepack {resource pack} shader {shader} plugin {plugin} datapack {data pack} server {server} other {project}}.',
	},
	avoidServerRepetition: {
		id: 'project.settings.description.guidance.avoid-server-repetition',
		defaultMessage:
			'Do NOT add repetitive or unnecessary information, such as joining instructions, unless unique to the server.',
	},
	avoidAi: {
		id: 'project.settings.description.guidance.avoid-ai',
		defaultMessage:
			'Do NOT rely on AI-generated text, as AI is likely to be inaccurate or create unappealing descriptions that users will avoid.',
	},
	guidanceRules: {
		id: 'project.settings.description.guidance.content-rules',
		defaultMessage:
			"The content of your description must abide by all of Modrinth's <rules>Content Rules</rules>.",
	},
	updated: { id: 'project.settings.description.updated', defaultMessage: 'Description updated' },
	updatedText: {
		id: 'project.settings.description.updated-text',
		defaultMessage: 'Your description has been updated.',
	},
})
const guidelines = computed(() => [
	messages.explainAppeal,
	...(isServerProject.value ? [] : [messages.explainUse]),
	messages.showGallery,
	messages.accessibleFormatting,
])
const mistakes = computed(() => [
	isServerProject.value ? messages.avoidServerAssumptions : messages.avoidAssumptions,
	messages.avoidMisleadingClaims,
	isServerProject.value ? messages.avoidServerRepetition : messages.avoidRepetition,
	messages.avoidAi,
])

const aiImageWarningModal = useTemplateRef('aiImageWarningModal')

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
