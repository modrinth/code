<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" :closable="!loading" no-padding>
		<div class="flex max-w-[500px] flex-col gap-6 p-6">
			<Admonition
				:type="variant === 'loader-change' ? 'critical' : 'warning'"
				:header="
					variant === 'loader-change'
						? formatMessage(messages.loaderChangeTitle)
						: formatMessage(messages.gameVersionWarningTitle)
				"
			>
				<div class="flex flex-col gap-3">
					<span>
						{{
							variant === 'loader-change'
								? formatMessage(messages.loaderChangeBody)
								: formatMessage(messages.gameVersionWarningBody)
						}}
					</span>
					<div v-if="variant === 'loader-change'">
						<ButtonStyled color="red">
							<button :disabled="loading" @click="handleResetServer">
								<TrashIcon class="size-5" />
								{{ formatMessage(commonMessages.resetServerButton) }}
							</button>
						</ButtonStyled>
					</div>
				</div>
			</Admonition>

			<InlineBackupCreator
				ref="backupCreator"
				:backup-name="
					variant === 'loader-change' ? 'Before loader change' : 'Before version change'
				"
				hide-shift-click-hint
				@update:buttons-disabled="buttonsDisabled = $event"
			/>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled>
					<button :disabled="loading" @click="handleCancel">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<template v-if="variant === 'game-version-change'">
					<ButtonStyled>
						<button :disabled="buttonsDisabled || loading" @click="handleDisableConflicts">
							<SpinnerIcon
								v-if="loading && loadingAction === 'disable-conflicts'"
								class="size-5 animate-spin"
							/>
							<PowerOffIcon v-else class="size-5" />
							{{ formatMessage(messages.disableConflictsButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="orange">
						<button :disabled="buttonsDisabled || loading" @click="handleAutoFix">
							<SpinnerIcon
								v-if="loading && loadingAction === 'auto-fix'"
								class="size-5 animate-spin"
							/>
							<HammerIcon v-else class="size-5" />
							{{ formatMessage(messages.autoFixButton) }}
						</button>
					</ButtonStyled>
				</template>
				<template v-else>
					<ButtonStyled color="red">
						<button :disabled="buttonsDisabled || loading" @click="handleConfirmLoaderChange">
							<SpinnerIcon v-if="loading" class="size-5 animate-spin" />
							<CircleAlertIcon v-else class="size-5" />
							{{ formatMessage(messages.changeLoaderButton) }}
						</button>
					</ButtonStyled>
				</template>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import {
	CircleAlertIcon,
	HammerIcon,
	PowerOffIcon,
	SpinnerIcon,
	TrashIcon,
	XIcon,
} from '@modrinth/assets'
import { ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import InlineBackupCreator from '../../content-tab/components/modals/InlineBackupCreator.vue'

defineProps<{
	variant: 'loader-change' | 'game-version-change'
	loading?: boolean
}>()

const emit = defineEmits<{
	'confirm-loader-change': []
	'auto-fix': []
	'disable-conflicts': []
	'reset-server': []
	cancel: []
}>()

const { formatMessage } = useVIntl()

const modal = ref<InstanceType<typeof NewModal>>()
const buttonsDisabled = ref(false)
const loadingAction = ref<'auto-fix' | 'disable-conflicts' | null>(null)

function show(e?: MouseEvent) {
	loadingAction.value = null
	modal.value?.show(e)
}

function hide() {
	modal.value?.hide()
}

function handleCancel() {
	hide()
	emit('cancel')
}

function handleConfirmLoaderChange() {
	emit('confirm-loader-change')
}

function handleAutoFix() {
	loadingAction.value = 'auto-fix'
	emit('auto-fix')
}

function handleDisableConflicts() {
	loadingAction.value = 'disable-conflicts'
	emit('disable-conflicts')
}

function handleResetServer() {
	hide()
	emit('reset-server')
}

const messages = defineMessages({
	header: {
		id: 'installation-settings.incompatible-content.header',
		defaultMessage: 'Incompatible content installed',
	},
	loaderChangeTitle: {
		id: 'installation-settings.incompatible-content.loader-change-title',
		defaultMessage: 'Changing loaders is destructive',
	},
	loaderChangeBody: {
		id: 'installation-settings.incompatible-content.loader-change-body',
		defaultMessage:
			'When changing the loader, all installed content will be disabled. We recommend resetting your server instead.',
	},
	gameVersionWarningTitle: {
		id: 'installation-settings.incompatible-content.game-version-warning-title',
		defaultMessage: 'Incompatibility warning',
	},
	gameVersionWarningBody: {
		id: 'installation-settings.incompatible-content.game-version-warning-body',
		defaultMessage:
			'When changing the game version, we can either disable incompatible installed content or attempt to resolve the incompatibilities.',
	},
	changeLoaderButton: {
		id: 'installation-settings.incompatible-content.change-loader-button',
		defaultMessage: 'Change loader',
	},
	autoFixButton: {
		id: 'installation-settings.incompatible-content.auto-fix-button',
		defaultMessage: 'Auto-fix',
	},
	disableConflictsButton: {
		id: 'installation-settings.incompatible-content.disable-conflicts-button',
		defaultMessage: 'Disable conflicts',
	},
})

defineExpose({ show, hide })
</script>
