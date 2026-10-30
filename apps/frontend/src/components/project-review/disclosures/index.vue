<template>
	<div v-if="project" class="flex h-full min-h-0 flex-col gap-1 overflow-hidden">
		<div class="shrink-0 p-2">
			<ReviewPanel mode="inline" :target="{ kind: 'disclosures' }" hide-title />
		</div>
		<p v-if="isPending" role="status">Loading disclosures…</p>
		<div v-else-if="isError" role="alert">
			<p>Could not load disclosures.</p>
			<Button @click="refetch()">Retry</Button>
		</div>
		<EmptyState
			v-else-if="!canEditDisclosures"
			type="no-documents"
			heading="Upload versions before adding disclosures"
			description="Come back once you've uploaded your versions."
		/>
		<template v-else>
			<div class="min-h-0 flex-1 overflow-y-auto overscroll-contain px-2 pb-2 pt-3">
				<DisclosureCards
					:editor="disclosures"
					:project-title="project.name"
					variant="review"
					hide-description
				/>
			</div>
			<div class="z-10 mt-auto shrink-0">
				<p v-if="saveError" role="alert" class="text-red">
					Could not save all disclosure changes. Your edits have been kept. Try saving again.
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
import { Button, EmptyState, UnsavedChangesPopup } from '@modrinth/ui'

import DisclosureCards from '~/components/ui/project-settings/disclosures/DisclosureCards.vue'
import { injectProjectReviewPageContext } from '~/providers/project-review'

import ReviewPanel from '../review-panel/index.vue'

const { project, disclosures } = injectProjectReviewPageContext()
const {
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
