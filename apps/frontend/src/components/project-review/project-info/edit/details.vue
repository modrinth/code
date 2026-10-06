<template>
	<EditModal
		ref="modal"
		:section="formatMessage(reviewMessages.details)"
		:saving="saving"
		:can-save="canSave"
		width="32rem"
		@cancel="reset"
		@save="save"
	>
		<div class="flex flex-col gap-6">
			<div class="flex flex-col gap-2">
				<label id="project-review-visibility-label" class="font-semibold text-contrast">
					{{ formatMessage(reviewMessages.visibility) }}
				</label>
				<Combobox
					v-model="visibility"
					:options="visibilityOptions"
					:disabled="saving"
					aria-labelledby="project-review-visibility-label"
					trigger-type="base"
				/>
			</div>
			<div v-if="!project?.minecraft_server" class="flex flex-col gap-2">
				<label id="project-review-monetization-label" class="font-semibold text-contrast">
					{{ formatMessage(reviewMessages.monetization) }}
				</label>
				<Combobox
					v-model="monetization"
					:options="monetizationOptions"
					:disabled="saving"
					aria-labelledby="project-review-monetization-label"
					trigger-type="base"
				/>
			</div>
		</div>
	</EditModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { Combobox, defineMessages, useVIntl } from '@modrinth/ui'
import { computed, ref, useTemplateRef } from 'vue'

import { projectReviewMessages as reviewMessages } from '../../messages'
import EditModal from './edit-modal.vue'
import { useProjectInfoEdit } from './use-project-edit'

type Visibility = 'approved' | 'unlisted' | 'private'
type MonetizationStatus = Labrinth.Projects.v2.MonetizationStatus

const messages = defineMessages({
	publicDescription: {
		id: 'moderation.project-review.details.publicDescription',
		defaultMessage: 'Visible via URL, on your profile, and in search.',
	},
	unlistedDescription: {
		id: 'moderation.project-review.details.unlistedDescription',
		defaultMessage: 'Visible via URL only. Not shown on your profile or in search.',
	},
	privateDescription: {
		id: 'moderation.project-review.details.privateDescription',
		defaultMessage: 'Not publicly visible. Only accessible to project members.',
	},
})

const { formatMessage } = useVIntl()
const { project, saving, beginEditing, saveProject } = useProjectInfoEdit()
const modal = useTemplateRef<InstanceType<typeof EditModal>>('modal')
const visibility = ref<Visibility>('approved')
const monetization = ref<MonetizationStatus>('monetized')
const hasPublishedStatus = computed(() =>
	['approved', 'archived', 'unlisted', 'private'].includes(project.value?.status ?? ''),
)
const currentVisibility = computed<Visibility>(() => {
	const status = hasPublishedStatus.value ? project.value?.status : project.value?.requested_status
	return status === 'unlisted' || status === 'private' ? status : 'approved'
})
const visibilityOptions = computed(() =>
	(['approved', 'unlisted', 'private'] as const).map((value) => ({
		value,
		label: formatMessage(reviewMessages.visibilityOption, { status: value }),
		subLabel: formatMessage(
			value === 'approved'
				? messages.publicDescription
				: value === 'unlisted'
					? messages.unlistedDescription
					: messages.privateDescription,
		),
	})),
)
const monetizationOptions = computed(() =>
	(['monetized', 'demonetized', 'force-demonetized'] as const).map((value) => ({
		value,
		label: formatMessage(reviewMessages.monetizationStatus, {
			status: value === 'force-demonetized' ? 'forceDemonetized' : value,
		}),
	})),
)
const patch = computed<Labrinth.Projects.v3.EditProjectRequest>(() => {
	const result: Labrinth.Projects.v3.EditProjectRequest = {}
	if (visibility.value !== currentVisibility.value) {
		if (hasPublishedStatus.value) result.status = visibility.value
		else result.requested_status = visibility.value
	}
	if (
		!project.value?.minecraft_server &&
		monetization.value !== project.value?.monetization_status
	) {
		result.monetization_status = monetization.value
	}
	return result
})
const canSave = computed(
	() => !!project.value && !saving.value && Object.keys(patch.value).length > 0,
)

function reset() {
	visibility.value = currentVisibility.value
	monetization.value = project.value?.monetization_status ?? 'monetized'
}

function show() {
	if (!project.value || saving.value) return
	beginEditing()
	reset()
	modal.value?.show()
}

async function save() {
	if (!canSave.value) return
	if (await saveProject(patch.value)) modal.value?.hide()
}

defineExpose({ show })
</script>
