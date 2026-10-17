<script setup lang="ts">
import { UnknownIcon } from '@modrinth/assets'
import {
	defineMessages,
	IntlFormatted,
	normalizeChildren,
	SettingsInlineWarning,
	SettingsToggleCard,
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
		infoLink?: string
	}
>()

const emit = defineEmits<{
	setLockStatus: [status: DisclosureLockStatus]
}>()

const enabled = defineModel<boolean>({ required: true })

const resolvedLockStatus = computed(() => props.lockStatus ?? 'unlocked')

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
	<component
		:is="variant === 'review' ? ProjectReviewDisclosureCard : SettingsToggleCard"
		v-model="enabled"
		:disabled="disabled"
		:toggle-disabled="toggleDisabled"
		:icon="icon"
		:title="title"
		:description="hideDescription ? undefined : description"
	>
		<template v-if="!hideDescription && $slots.default" #default>
			<slot />
		</template>
		<template #title-suffix>
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
		</template>
		<template v-if="$slots.expanded" #expanded>
			<slot name="expanded" />
		</template>
		<template v-if="variant === 'review' && updatedAt" #updated-by>
			<DisclosureUpdatedBy
				:updated-at="updatedAt"
				:updated-by="updatedBy"
				:set-by-moderator="setByModerator"
			/>
		</template>
		<template v-if="variant === 'review' && showModeratorLockControls" #lock-controls>
			<DisclosureLockControls
				:lock-status="resolvedLockStatus"
				:disabled="disabled"
				@set-lock-status="setLockStatus"
			/>
		</template>
		<template v-if="variant !== 'review' && showFooter" #footer>
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
	</component>
</template>
