<script setup lang="ts">
import { DropdownIcon, InfoIcon, UnknownIcon } from '@modrinth/assets'
import {
	defineMessages,
	IntlFormatted,
	normalizeChildren,
	SettingsInlineWarning,
	ToggleCard,
	useVIntl,
} from '@modrinth/ui'
import type { Component } from 'vue'
import { computed } from 'vue'

import DisclosureLockControls from './DisclosureLockControls.vue'
import DisclosureUpdatedBy from './DisclosureUpdatedBy.vue'
import ProjectReviewDisclosureCard from './ProjectReviewDisclosureCard.vue'
import type { DisclosureCardMetaProps, DisclosureLockStatus } from './types'

defineOptions({ inheritAttrs: false })

const { formatMessage } = useVIntl()

const props = defineProps<
	DisclosureCardMetaProps & {
		title: string
		icon?: Component
		description?: string
		guidanceTitle?: string
		infoLink?: string
	}
>()

const emit = defineEmits<{
	setLockStatus: [status: DisclosureLockStatus]
}>()

const enabled = defineModel<boolean>({ required: true })

const resolvedLockStatus = computed(() => props.lockStatus ?? 'unlocked')
const showGuidance = computed(
	() => !props.hideDescription && !!props.guidanceTitle,
)

const messages = defineMessages({
	cannotDisableWarning: {
		id: 'project.settings.disclosures.lock-status.cannot-disable.warning',
		defaultMessage:
			'Please <contact-support-link>contact support</contact-support-link> if this disclosure needs to be removed.',
	},
	fullyLockedWarning: {
		id: 'project.settings.disclosures.lock-status.fully-locked.warning',
		defaultMessage:
			'Please <contact-support-link>contact support</contact-support-link> if this disclosure needs to be edited or removed.',
	},
	disabledLockedWarning: {
		id: 'project.settings.disclosures.lock-status.disabled.warning',
		defaultMessage:
			'This disclosure has been locked by the moderators. Please <contact-support-link>contact support</contact-support-link> if you believe this is in error.',
	},
	infoTooltip: {
		id: 'project.settings.disclosures.info-tooltip',
		defaultMessage: 'Click to learn more about when to use this disclosure.',
	},
})

const lockWarningMessage = computed(() => {
	if (resolvedLockStatus.value === 'cannot_disable') {
		return enabled.value ? messages.cannotDisableWarning : null
	}
	if (resolvedLockStatus.value === 'fully_locked') {
		return enabled.value ? messages.fullyLockedWarning : messages.disabledLockedWarning
	}
	return null
})

const showLockWarning = computed(() => !!lockWarningMessage.value)

const showModeratorLockControls = computed(() => !!props.showLockControls)

const showFooter = computed(
	() => !!props.updatedAt || showModeratorLockControls.value || showLockWarning.value,
)

function setLockStatus(status: DisclosureLockStatus) {
	if (resolvedLockStatus.value === status) return
	emit('setLockStatus', status)
}
</script>

<template>
	<ProjectReviewDisclosureCard
		v-if="variant === 'review'"
		v-model="enabled"
		:disabled="disabled"
		:toggle-disabled="toggleDisabled"
		:editing="editing"
		:on-toggle-editing="onToggleEditing"
		:on-cancel-editing="onCancelEditing"
		:on-save-editing="onSaveEditing"
		:section-can-save="sectionCanSave"
		:section-saving="sectionSaving"
		:section-save-error="sectionSaveError"
		:section-save-disabled-reason="sectionSaveDisabledReason"
		:icon="icon"
		:title="title"
		:description="hideDescription ? undefined : description"
	>
		<template v-if="!hideDescription && $slots.default" #default>
			<slot />
		</template>
		<template v-if="$slots.expanded" #expanded>
			<slot name="expanded" />
		</template>
		<template v-if="updatedAt" #updated-by>
			<DisclosureUpdatedBy
				:updated-at="updatedAt"
				:updated-by="updatedBy"
				:set-by-moderator="setByModerator"
			/>
		</template>
		<template v-if="showModeratorLockControls" #lock-controls>
			<DisclosureLockControls
				compact
				:lock-status="resolvedLockStatus"
				:disabled="disabled"
				@set-lock-status="setLockStatus"
			/>
		</template>
	</ProjectReviewDisclosureCard>
	<ToggleCard v-else v-model="enabled" :disabled="disabled" :toggle-disabled="toggleDisabled">
		<h3 class="mb-1 mt-0 flex items-center gap-2 text-lg font-semibold text-contrast">
			<component :is="icon" v-if="icon" class="size-5 shrink-0 text-primary" aria-hidden="true" />
			{{ title }}
			<a
				v-if="infoLink"
				v-tooltip="formatMessage(messages.infoTooltip)"
				class="smart-clickable:allow-pointer-events flex text-secondary hover:brightness-[--hover-brightness]"
				:href="infoLink"
				target="_blank"
				rel="noopener noreferrer"
			>
				<UnknownIcon class="size-5" />
			</a>
		</h3>
		<div
			v-if="!hideDescription && (description || (!showGuidance && $slots.default))"
			class="flex flex-col gap-2 leading-normal [&>p]:m-0"
		>
			<p v-if="description">{{ description }}</p>
			<slot v-if="!showGuidance" />
		</div>
		<template v-if="showGuidance && $slots.default" #after-summary>
			<details class="disclosure-guidance group/guidance">
				<summary
					class="relative isolate flex cursor-pointer select-none list-none items-center gap-2 px-4 py-3 font-medium text-primary before:pointer-events-none before:absolute before:inset-0 before:-z-10 before:bg-surface-3 before:transition-[filter] before:duration-150 before:content-[''] hover:before:[filter:brightness(1.15)] focus-visible:-outline-offset-2 motion-reduce:before:transition-none [&::-webkit-details-marker]:hidden"
				>
					<InfoIcon class="size-5 shrink-0 text-secondary" aria-hidden="true" />
					<span class="min-w-0 flex-1 pt-0.5">{{ guidanceTitle }}</span>
					<DropdownIcon
						class="size-5 shrink-0 text-secondary group-open/guidance:rotate-180"
						aria-hidden="true"
					/>
				</summary>
				<div class="flex flex-col gap-3 px-4 pb-4 leading-normal text-secondary [&>p]:m-0">
					<slot />
				</div>
			</details>
		</template>
		<template v-if="$slots.expanded" #expanded>
			<div class="flex flex-col gap-4">
				<slot name="expanded" />
			</div>
		</template>
		<template v-if="showFooter" #footer>
			<div class="flex flex-col gap-3">
				<SettingsInlineWarning v-if="showLockWarning && lockWarningMessage">
					<IntlFormatted :message-id="lockWarningMessage">
						<template #contact-support-link="{ children }">
							<a
								class="smart-clickable:allow-pointer-events text-orange underline hover:brightness-110"
								href="https://support.modrinth.com"
								target="_blank"
								rel="noopener noreferrer"
							>
								<component :is="() => normalizeChildren(children)" />
							</a>
						</template>
					</IntlFormatted>
				</SettingsInlineWarning>
				<div class="flex flex-wrap items-center justify-between gap-3">
					<DisclosureUpdatedBy
						v-if="updatedAt"
						:updated-at="updatedAt"
						:updated-by="updatedBy"
						:set-by-moderator="setByModerator"
					/>
					<span v-else />
					<DisclosureLockControls
						v-if="showModeratorLockControls"
						class="ml-auto"
						:lock-status="resolvedLockStatus"
						:disabled="disabled"
						@set-lock-status="setLockStatus"
					/>
				</div>
			</div>
		</template>
	</ToggleCard>
</template>

<style scoped>
@media (prefers-reduced-motion: no-preference) {
	@supports (interpolate-size: allow-keywords) and selector(details::details-content) {
		.disclosure-guidance::details-content {
			block-size: 0;
			overflow: clip;
			transition:
				block-size 200ms ease-in-out,
				content-visibility 200ms allow-discrete;
		}

		.disclosure-guidance[open]::details-content {
			block-size: auto;
		}
	}
}
</style>
