<script setup lang="ts">
import { computed, ref } from 'vue'

import AutoLink from '#ui/components/base/AutoLink.vue'
import Checkbox from '#ui/components/base/Checkbox.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { truncatedTooltip } from '#ui/utils/truncate'

import type { ContentCardTableItem, ContentSide } from '../../types'
import ContentEnabledFor from '../ContentEnabledFor.vue'
import ContentItemSummary from '../ContentItemSummary.vue'
import RowActions from './row-actions.vue'
import type { ContentTableAction, ContentTableLayout } from './types'

const props = defineProps<{
	row: ContentCardTableItem
	layout: ContentTableLayout
	actions: Set<ContentTableAction>
	selection: boolean
	selected: boolean
	showVersion: boolean
	showEnabledFor: boolean
	showActions: boolean
}>()

const emit = defineEmits<{
	select: [value: boolean, event?: MouseEvent]
	toggle: [value: boolean]
	enabledFor: [side: ContentSide, value: boolean]
	remove: [event: MouseEvent]
	update: []
	switchVersion: []
	prefetch: []
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	selectProject: {
		id: 'content.card.select-project',
		defaultMessage: 'Select {project}',
	},
})

const versionNumberRef = ref<HTMLElement | null>(null)
const fileNameRef = ref<HTMLElement | null>(null)

const stacked = computed(() => props.layout === 'stacked' || props.layout === 'narrow')
const separateVersion = computed(() => props.showVersion && props.layout === 'wide')
const title = computed(() => props.row.project.title.replace(/§[0-9a-fk-orx]/gi, ''))
const isDisabled = computed(() => !!props.row.disabled || !!props.row.installing)
const dimmed = computed(() => props.row.enabled === false && !isDisabled.value)

function linkTarget(link: unknown) {
	return typeof link === 'string' && link.startsWith('http') ? '_blank' : undefined
}

function splitHalves(value: string) {
	const middle = Math.ceil(value.length / 2)
	return [value.slice(0, middle), value.slice(middle)]
}
</script>

<template>
	<div
		role="row"
		class="grid h-[var(--content-row-height)] grid-cols-[var(--content-columns)] items-center gap-x-4 px-3"
		:class="[
			{ 'gap-y-2 py-3': stacked },
			{ 'opacity-50 grayscale': row.disabled && !row.installing, 'opacity-50': row.installing },
		]"
	>
		<div class="flex min-w-0 items-center gap-4" :class="{ 'col-span-full': stacked }">
			<Checkbox
				v-if="selection"
				:model-value="selected"
				:aria-label="formatMessage(messages.selectProject, { project: title })"
				:disabled="isDisabled"
				class="shrink-0"
				@update:model-value="(value, event) => emit('select', value, event)"
			/>
			<ContentItemSummary
				:project="row.project"
				:project-link="row.projectLink"
				:version="row.version"
				:version-link="row.versionLink"
				:show-version="showVersion && !separateVersion"
				:owner="row.owner"
				:source="row.source"
				:external="row.external"
				:external-file="row.externalFile"
				:override-file="row.overrideFile"
				:installing="row.installing"
				:install-progress="row.installProgress"
				:client-warning="row.clientWarning"
				:synced-badge="!!row.synced && !showActions"
				:sync-update-pending="row.syncUpdatePending"
				:embedded-icon="row.embeddedIcon"
				:dimmed="dimmed"
			>
				<template #title-badges>
					<slot name="title-badges" />
				</template>
			</ContentItemSummary>
		</div>

		<div v-if="showEnabledFor" class="min-w-0">
			<ContentEnabledFor
				v-if="row.enabledFor"
				reserve-status-space
				:model-value="row.enabledFor"
				:loading="row.installing"
				:disabled="isDisabled"
				:disabled-tooltip="isDisabled ? (row.disabledTooltip ?? undefined) : undefined"
				@update:model-value="(side, value) => emit('enabledFor', side, value)"
			/>
		</div>

		<div
			v-if="separateVersion"
			class="flex min-w-0 flex-col gap-0.5 transition-[filter,opacity] duration-200"
			:class="{ 'grayscale opacity-50': dimmed }"
		>
			<template v-if="row.version">
				<AutoLink
					v-if="row.version.version_number"
					v-tooltip="truncatedTooltip(versionNumberRef, row.version.version_number)"
					:target="linkTarget(row.versionLink)"
					:to="row.versionLink"
					class="inline-flex min-w-0 font-semibold leading-6 text-contrast !decoration-contrast"
					:class="{ 'hover:underline cursor-pointer': row.versionLink }"
				>
					<span ref="versionNumberRef" class="min-w-0 truncate">{{
						splitHalves(row.version.version_number)[0]
					}}</span
					><span class="min-w-0 max-w-[50%] truncate">{{
						splitHalves(row.version.version_number)[1]
					}}</span>
				</AutoLink>
				<span
					v-tooltip="truncatedTooltip(fileNameRef, row.version.file_name)"
					class="flex min-w-0 leading-6 text-secondary"
				>
					<span ref="fileNameRef" class="min-w-0 truncate">{{
						splitHalves(row.version.file_name)[0]
					}}</span
					><span class="min-w-0 max-w-[50%] truncate">{{
						splitHalves(row.version.file_name)[1]
					}}</span>
				</span>
			</template>
		</div>

		<RowActions
			v-if="showActions"
			:class="layout === 'narrow' ? 'justify-self-start' : 'justify-self-end'"
			:row="row"
			:actions="actions"
			:title="title"
			:disabled="isDisabled"
			@toggle="(value) => emit('toggle', value)"
			@remove="(event) => emit('remove', event)"
			@update="emit('update')"
			@switch-version="emit('switchVersion')"
			@prefetch="emit('prefetch')"
		>
			<template #before>
				<slot name="actions-before" />
			</template>
			<template #after>
				<slot name="actions-after" />
			</template>
		</RowActions>
	</div>
</template>
