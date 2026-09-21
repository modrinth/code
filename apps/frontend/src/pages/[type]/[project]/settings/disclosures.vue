<script setup lang="ts">
import {
	Button,
	commonMessages,
	ConfirmLeaveModal,
	EmptyState,
	injectProjectPageContext,
	IntlFormatted,
	normalizeChildren,
	UnsavedChangesPopup,
	usePageLeaveSafety,
	useVIntl,
} from '@modrinth/ui'
import { isAdmin, isStaff, TeamMemberPermission } from '@modrinth/utils'
import { computed } from 'vue'

import { disclosureStatusMessages as statusMessages } from '~/components/ui/project-settings/disclosures/messages'
import { useDisclosureEditor } from '~/components/ui/project-settings/disclosures/use-disclosure-editor'
import ValidationMessage from '~/components/ValidationMessage.vue'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { useProjectSaveValidation } from '~/composables/project-save-validation'

const {
	projectV2: project,
	projectV3,
	currentMember,
	refreshProjectValidation,
} = injectProjectPageContext()
const flags = useFeatureFlags()
const editor = useDisclosureEditor({
	projectId: computed(() => project.value.id),
	projectTypes: computed(() => projectV3.value?.project_types ?? [project.value.project_type]),
	canEditDisclosures: computed(
		() => project.value.versions.length > 0 || projectV3.value?.minecraft_server != null,
	),
	hasPermission: computed(
		() =>
			isAdmin(currentMember.value?.user) ||
			!!((currentMember.value?.permissions ?? 0) & TeamMemberPermission.EDIT_DETAILS),
	),
	isActingAsModerator: computed(
		() => isStaff(currentMember.value?.user) && !flags.value.showModeratorProjectMemberUi,
	),
	isAdminUser: computed(() => isAdmin(currentMember.value?.user)),
	onSaved: refreshProjectValidation,
})
useProjectSettingsHeadTitle(editor.messages.title)
const disclosureTextValidation = useProjectNagMessages('disclosure-text')
const disclosureValidation = useProjectNagMessages('disclosures')
const { confirmLeaveModal } = usePageLeaveSafety(editor.hasChanges)

const { formatMessage } = useVIntl()
const {
	messages,
	savedSnapshot,
	currentSnapshot,
	saving,
	reset,
	save,
	canSave,
	saveDisabledReason,
	canEditDisclosures,
	disclosuresQuery,
	saveError,
} = editor
const { isPending, isError, refetch } = disclosuresQuery
</script>

<template>
	<div>
		<ConfirmLeaveModal ref="confirmLeaveModal" />
		<h2 class="m-0 text-2xl font-semibold">{{ formatMessage(messages.title) }}</h2>
		<p class="mb-0 mt-2">
			<IntlFormatted :message-id="messages.description">
				<template #rules="{ children }">
					<nuxt-link to="/legal/rules" target="_blank" class="underline hover:text-contrast">
						<component :is="() => normalizeChildren(children)" />
					</nuxt-link>
				</template>
			</IntlFormatted>
		</p>
		<p class="mb-4 mt-2">
			<IntlFormatted :message-id="messages.description2">
				<template #faq-link="{ children }">
					<a
						href="https://support.modrinth.com/en/articles/16567675#h_29503820b1"
						target="_blank"
						class="underline hover:text-contrast"
					>
						<component :is="() => normalizeChildren(children)" />
					</a>
				</template>
			</IntlFormatted>
		</p>
		<ValidationMessage
			class="mb-4"
			:check="[...disclosureValidation, ...disclosureTextValidation]"
			:project-field="JSON.stringify(savedSnapshot)"
			:current-field="JSON.stringify(currentSnapshot)"
		/>
		<p v-if="isPending" role="status">{{ formatMessage(statusMessages.loading) }}</p>
		<div v-else-if="isError" role="alert">
			<p>{{ formatMessage(statusMessages.loadError) }}</p>
			<Button @click="refetch()">{{ formatMessage(statusMessages.retry) }}</Button>
		</div>
		<EmptyState
			v-else-if="!canEditDisclosures"
			type="no-documents"
			:heading="formatMessage(messages.uploadVersionFirstHeading)"
			:description="formatMessage(commonMessages.uploadVersionsEmptyStateDescription)"
		/>
		<template v-else>
			<DisclosureCards :editor="editor" :project-title="project.title" />
			<div>
				<p v-if="saveError" role="alert" class="text-red">
					{{ formatMessage(statusMessages.saveError) }}
				</p>
				<UnsavedChangesPopup
					:original="savedSnapshot"
					:modified="currentSnapshot"
					:saving="saving"
					:can-save="canSave"
					:save-disabled-reason="saveDisabledReason"
					@reset="reset"
					@save="save"
				/>
			</div>
		</template>
	</div>
</template>
