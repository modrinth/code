<template>
	<Section :heading="formatMessage(messages.license)" :target="{ kind: 'license' }">
		<template #heading-after>
			<TagItem v-if="isCustomLicense" class="!border-surface-4 !bg-surface-3 !text-secondary">
				Custom
			</TagItem>
		</template>
		<template #right>
			<EditButton :section="formatMessage(messages.license)" @click="editModal?.show()" />
		</template>
		<EditModal :key="project?.id" ref="editModal" />
		<div v-if="project" class="flex flex-col gap-1">
			<div class="flex items-center gap-2">
				<span
					v-tooltip="project.license.name || projectV2?.license.name"
					class="flex min-h-6 min-w-0 items-center"
				>
					{{
						project.license.id === 'LicenseRef-All-Rights-Reserved'
							? 'All-Rights-Reserved'
							: project.license.id
					}}
				</span>
				<TagItem
					v-tooltip="formatMessage(messages.sourceAvailabilityRequired)"
					class="!border-orange !bg-highlight-orange !px-1.5 !py-0.5 !text-secondary"
				>
					<TagCategoryScrollTextIcon />
				</TagItem>
			</div>
			<div>
				<a
					v-if="url"
					v-tooltip="url"
					:href="url"
					target="_blank"
					rel="noopener noreferrer"
					class="mt-1 flex w-fit min-w-0 max-w-full items-center gap-1 font-mono text-xs !transition-colors hover:text-contrast"
				>
					<span class="min-w-0 truncate">{{ url.replace(/^https?:\/\//, '') }}</span>
					<ExternalIcon class="size-3 shrink-0" />
				</a>
			</div>
		</div>
	</Section>
</template>

<script setup lang="ts">
import { ExternalIcon, TagCategoryScrollTextIcon } from '@modrinth/assets'
import { licenseRequiresSource } from '@modrinth/moderation'
import { TagItem, useVIntl } from '@modrinth/ui'
import { computed, useTemplateRef } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'
import { reviewExternalUrl } from '~/providers/project-review/project-links'

import { projectReviewMessages as messages } from '../../messages'
import EditButton from '../edit/button.vue'
import EditModal from '../edit/license.vue'
import Section from '../section.vue'

const { project, projectV2 } = injectProjectReviewPageContext()
const { formatMessage } = useVIntl()
const editModal = useTemplateRef<InstanceType<typeof EditModal>>('editModal')
const isCustomLicense = computed(() => {
	const id = project.value?.license.id
	return (
		!!id &&
		id.startsWith('LicenseRef-') &&
		id !== 'LicenseRef-All-Rights-Reserved' &&
		id !== 'LicenseRef-Unknown' &&
		id !== 'LicenseRef-NOASSERTION'
	)
})
const url = computed(() => reviewExternalUrl(project.value?.license.url))
</script>
