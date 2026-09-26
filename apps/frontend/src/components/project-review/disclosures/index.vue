<template>
	<div v-if="project" class="flex h-full min-h-0 flex-col gap-1 overflow-hidden">
		<ReviewPanel mode="inline" :target="{ kind: 'disclosures' }" />
		<p v-if="isPending" role="status">{{ formatMessage(statusMessages.loading) }}</p>
		<div v-else-if="isError" role="alert">
			<p>{{ formatMessage(statusMessages.loadError) }}</p>
			<Button @click="refetch()">{{ formatMessage(statusMessages.retry) }}</Button>
		</div>
		<EmptyState
			v-else-if="!canEditDisclosures"
			type="no-documents"
			:heading="formatMessage(editorMessages.uploadVersionFirstHeading)"
			:description="formatMessage(commonMessages.uploadVersionsEmptyStateDescription)"
		/>
		<template v-else>
			<div class="min-h-0 flex-1 overflow-y-auto overscroll-contain px-1 pb-1 pt-3">
				<DisclosureCards
					:editor="disclosures"
					:project-title="project.name"
					variant="review"
					hide-description
				/>
			</div>
			<div class="z-10 mt-auto shrink-0">
				<p v-if="saveError" role="alert" class="text-red">
					{{ formatMessage(statusMessages.saveError) }}
				</p>
				<UnsavedChangesPopup
					inline
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

<script setup lang="ts">
import { Button, commonMessages, EmptyState, UnsavedChangesPopup, useVIntl } from '@modrinth/ui'

import DisclosureCards from '~/components/ui/project-settings/disclosures/DisclosureCards.vue'
import { disclosureStatusMessages as statusMessages } from '~/components/ui/project-settings/disclosures/messages'
import { injectProjectReviewPageContext } from '~/providers/project-review'

import ReviewPanel from '../review-panel/index.vue'

const { formatMessage } = useVIntl()
const { project, disclosures } = injectProjectReviewPageContext()
const {
	messages: editorMessages,
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
} = disclosures
const { isPending, isError, refetch } = disclosuresQuery
</script>
