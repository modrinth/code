<script setup lang="ts">
import { CircleSlashIcon, LockIcon, LockOpenIcon } from '@modrinth/assets'
import {
	Button,
	ButtonGroup,
	defineMessages,
	IntlFormatted,
	normalizeChildren,
	SettingsInlineWarning,
	SettingsToggleCard,
} from '@modrinth/ui'
import type { Component } from 'vue'
import { computed } from 'vue'

import DisclosureUpdatedBy from './DisclosureUpdatedBy.vue'
import type { DisclosureCardMetaProps, DisclosureLockStatus } from './types'

defineOptions({ inheritAttrs: false })

const props = defineProps<
	DisclosureCardMetaProps & {
		title: string
		icon?: Component
		description?: string
	}
>()

const emit = defineEmits<{
	setLockStatus: [status: DisclosureLockStatus]
}>()

const enabled = defineModel<boolean>({ required: true })

const resolvedLockStatus = computed(() => props.lockStatus ?? 'unlocked')

const LOCK_STATUSES = [
	{ status: 'unlocked', label: 'Unlocked' },
	{ status: 'cannot_disable', label: 'Cannot disable' },
	{ status: 'fully_locked', label: 'Fully locked' },
] as const

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
	<SettingsToggleCard
		v-model="enabled"
		:disabled="disabled"
		:toggle-disabled="toggleDisabled"
		:icon="icon"
		:title="title"
		:description="description"
	>
		<slot />
		<template v-if="$slots.expanded" #expanded>
			<slot name="expanded" />
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
					<ButtonGroup v-if="showModeratorLockControls" class="ml-auto" label="Lock status">
						<Button
							v-for="{ status, label } in LOCK_STATUSES"
							:key="status"
							:color="
								status === 'cannot_disable'
									? 'orange'
									: status === 'fully_locked'
										? 'red'
										: undefined
							"
							:type="status !== 'unlocked' ? 'colored-text' : undefined"
							:disabled="disabled || resolvedLockStatus === status"
							@click="setLockStatus(status)"
						>
							<LockOpenIcon v-if="status === 'unlocked'" />
							<CircleSlashIcon v-else-if="status === 'cannot_disable'" />
							<LockIcon v-else />
							{{ label }}
						</Button>
					</ButtonGroup>
				</div>
			</div>
		</template>
	</SettingsToggleCard>
</template>
