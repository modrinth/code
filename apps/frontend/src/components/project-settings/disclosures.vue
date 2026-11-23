<script setup lang="ts">
import {
	Button,
	ConfirmLeaveModal,
	injectProjectPageContext,
	IntlFormatted,
	normalizeChildren,
	UnsavedChangesPopup,
	usePageLeaveSafety,
	useVIntl,
} from '@modrinth/ui'
import { isAdmin, isStaff, TeamMemberPermission } from '@modrinth/utils'
import { useQueryClient } from '@tanstack/vue-query'
import { computed } from 'vue'

import ProjectIssueCard from '~/components/ui/project-issue-card/index.vue'
import DisclosureCards from '~/components/ui/project-settings/disclosures/DisclosureCards.vue'
import { disclosureStatusMessages as statusMessages } from '~/components/ui/project-settings/disclosures/messages'
import { useDisclosureEditor } from '~/components/ui/project-settings/disclosures/use-disclosure-editor'
import ValidationMessage from '~/components/ValidationMessage.vue'
import { useProjectNagMessages } from '~/composables/project-nag-validation'

import SettingsSection from './settings-section.vue'

const {
	projectV2: project,
	projectV3,
	currentMember,
	refreshProjectValidation,
	thread,
} = injectProjectPageContext()
const flags = useFeatureFlags()
const queryClient = useQueryClient()
const editor = useDisclosureEditor({
	projectId: computed(() => project.value.id),
	projectTypes: computed(() => {
		if (project.value.versions.length === 0 && projectV3.value?.minecraft_server == null) {
			return null
		}
		return projectV3.value?.project_types ?? [project.value.project_type]
	}),
	hasPermission: computed(
		() =>
			isAdmin(currentMember.value?.user) ||
			!!((currentMember.value?.permissions ?? 0) & TeamMemberPermission.EDIT_DETAILS),
	),
	isActingAsModerator: computed(
		() => isStaff(currentMember.value?.user) && !flags.value.showModeratorProjectMemberUi,
	),
	isAdminUser: computed(() => isAdmin(currentMember.value?.user)),
	onSaved: async () => {
		await Promise.all([
			refreshProjectValidation(),
			thread.value
				? queryClient.invalidateQueries({ queryKey: ['thread', thread.value.id] })
				: Promise.resolve(),
		])
	},
})
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
	disclosuresQuery,
	saveError,
	issueAction,
} = editor
const { isPending, isError, refetch } = disclosuresQuery
</script>

<template>
	<div data-project-field="disclosures" data-project-field-scroll-only>
		<ConfirmLeaveModal ref="confirmLeaveModal" />
		<ProjectIssueCard
			location="disclosures"
			:field-action="issueAction"
			:target="[
				'add_project_disclosures',
				'remove_project_disclosures',
				'modify_project_disclosure',
				'modify_project_disclosure_note',
			]"
			class="mb-4"
		/>
		<SettingsSection :title="formatMessage(messages.title)" class="mb-4">
			<template #subtitle>
				<IntlFormatted :message-id="messages.description">
					<template #rules="{ children }">
						<nuxt-link
							to="/legal/rules"
							target="_blank"
							class="text-blue hover:underline focus-visible:underline"
						>
							<component :is="() => normalizeChildren(children)" />
						</nuxt-link>
					</template>
				</IntlFormatted>
				{{ ' ' }}
				<IntlFormatted :message-id="messages.description2">
					<template #faq-link="{ children }">
						<a
							href="https://support.modrinth.com/en/articles/16567675#h_29503820b1"
							target="_blank"
							class="text-blue hover:underline focus-visible:underline"
						>
							<component :is="() => normalizeChildren(children)" />
						</a>
					</template>
				</IntlFormatted>
			</template>
		</SettingsSection>
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
