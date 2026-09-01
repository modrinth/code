<script setup lang="ts">
import { UnknownIcon } from '@modrinth/assets'
import { Button, ButtonGroup, defineMessages, Input, Toggle, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import type { EditableGameSetting, GameOptionCanonicalValue } from '@/helpers/game-options'

import {
	canonicalBooleanValue,
	canonicalValueFromInput,
	canonicalValueText,
} from './game-setting-editors'
import {
	formatCompatibilitySubtitle,
	formatCompatibilityTooltip,
	formatGameSettingChoice,
	formatGameSettingDescription,
	formatGameSettingLabel,
	formatGameSettingValidation,
	presentationMessages,
	shouldShowCompatibilityIndicator,
} from './game-setting-messages'

const props = withDefaults(
	defineProps<{
		setting: EditableGameSetting
		disabled?: boolean
	}>(),
	{
		disabled: false,
	},
)

const emit = defineEmits<{
	'update:sync-enabled': [enabled: boolean]
	'update:canonical-value': [value: GameOptionCanonicalValue | null]
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	valueLabel: {
		id: 'app.settings.synced-options.game-settings.value-label',
		defaultMessage: 'Setting value',
	},
	on: {
		id: 'app.settings.synced-options.game-settings.value-on',
		defaultMessage: 'On',
	},
	off: {
		id: 'app.settings.synced-options.game-settings.value-off',
		defaultMessage: 'Off',
	},
	mixed: {
		id: 'app.settings.synced-options.game-settings.value-mixed',
		defaultMessage: 'Mixed',
	},
	unset: {
		id: 'app.settings.synced-options.game-settings.value-unset',
		defaultMessage: 'Choose a value',
	},
	savedNotSynced: {
		id: 'app.settings.synced-options.game-settings.saved-not-synced',
		defaultMessage: 'Saved, not synced',
	},
	compatibilityDetails: {
		id: 'app.settings.synced-options.game-settings.compatibility-details',
		defaultMessage: 'Compatibility details',
	},
	syncSetting: {
		id: 'app.settings.synced-options.game-settings.sync-setting',
		defaultMessage: 'Sync {setting}',
	},
})

const settingLabel = computed(() => formatGameSettingLabel(formatMessage, props.setting))
const settingDescription = computed(() =>
	formatGameSettingDescription(formatMessage, props.setting),
)
const valueText = computed(() => canonicalValueText(props.setting))
const booleanValue = computed(() => canonicalBooleanValue(props.setting))
const numberScale = computed(() => (props.setting.editor.unit === 'percent' ? 100 : 1))
const inputMin = computed(() =>
	props.setting.editor.min === null || props.setting.editor.min === undefined
		? undefined
		: props.setting.editor.min * numberScale.value,
)
const inputMax = computed(() =>
	props.setting.editor.max === null || props.setting.editor.max === undefined
		? undefined
		: props.setting.editor.max * numberScale.value,
)
const inputStep = computed(() =>
	props.setting.editor.step === null || props.setting.editor.step === undefined
		? undefined
		: props.setting.editor.step * numberScale.value,
)
const editorDisabled = computed(
	() => props.disabled || props.setting.controlled,
)
const syncToggleDisabled = computed(
	() =>
		props.disabled ||
		props.setting.controlled ||
		(!props.setting.sync_enabled &&
			(!!props.setting.validation_error ||
				['mixed', 'unset', 'invalid'].includes(props.setting.value_state) ||
				(props.setting.compatibility.total_participating > 0 &&
					props.setting.compatibility.will_receive === 0))),
)
const placeholder = computed(() => {
	if (props.setting.value_state === 'mixed') return formatMessage(messages.mixed)
	if (props.setting.value_state === 'unset') return formatMessage(messages.unset)
	if (props.setting.editor.type === 'external_raw') {
		return formatMessage(presentationMessages.customValuePlaceholder)
	}
	return undefined
})
const compatibilityTone = computed(() =>
	props.setting.compatibility.left_local > 0
		? 'text-orange'
		: 'text-secondary',
)
const compatibilityTooltip = computed(() =>
	formatCompatibilityTooltip(formatMessage, props.setting),
)
const compatibilitySubtitle = computed(() =>
	formatCompatibilitySubtitle(formatMessage, props.setting),
)
const validationMessage = computed(() =>
	formatGameSettingValidation(formatMessage, props.setting.validation_error),
)
const syncDisabledReason = computed(() => {
	if (validationMessage.value) return validationMessage.value
	if (props.setting.value_state === 'invalid') {
		return formatMessage(presentationMessages.validationInvalidValue)
	}
	if (props.setting.value_state === 'mixed' || props.setting.value_state === 'unset') {
		return formatMessage(presentationMessages.validationMissingValue)
	}
	if (props.setting.controlled) {
		return formatMessage(presentationMessages.bucketLauncherControlled, { count: 1 })
	}
	if (
		props.setting.compatibility.total_participating > 0 &&
		props.setting.compatibility.will_receive === 0
	) {
		return formatMessage(presentationMessages.compatibilityNone)
	}
	return undefined
})

function updateValue(value: string | number | boolean | undefined) {
	emit('update:canonical-value', canonicalValueFromInput(props.setting, value))
}

function updateSelectValue(event: Event) {
	updateValue((event.target as HTMLSelectElement).value)
}
</script>

<template>
	<div class="flex min-w-0 items-center gap-4 border-0 border-b border-solid border-surface-5 py-3">
		<div class="min-w-0 flex-1">
			<div class="flex items-center gap-2">
				<h3 class="m-0 truncate text-base font-semibold text-contrast">
					{{ settingLabel }}
				</h3>
				<span
					v-if="shouldShowCompatibilityIndicator(setting)"
					v-tooltip="compatibilityTooltip"
					tabindex="0"
					role="img"
					:aria-label="formatMessage(messages.compatibilityDetails)"
					class="flex shrink-0 rounded-md outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow"
					:class="compatibilityTone"
				>
					<UnknownIcon class="size-4" aria-hidden="true" />
				</span>
			</div>
			<p v-if="settingDescription" class="m-0 mt-0.5 text-sm text-primary">
				{{ settingDescription }}
			</p>
			<p class="m-0 mt-0.5 text-xs text-secondary">
				{{ validationMessage ?? compatibilitySubtitle }}
				<span v-if="!setting.sync_enabled">
					· {{ formatMessage(messages.savedNotSynced) }}
				</span>
			</p>
		</div>

		<div class="flex shrink-0 items-center gap-3">
			<span class="sr-only">{{ formatMessage(messages.valueLabel) }}</span>
			<ButtonGroup
				v-if="setting.editor.type === 'bool'"
				:label="formatMessage(messages.valueLabel)"
			>
				<Button
					size="sm"
					:type="booleanValue === true ? 'colored' : 'outlined'"
					:color="booleanValue === true ? 'brand' : undefined"
					:disabled="editorDisabled"
					:aria-pressed="booleanValue === true"
					@click="updateValue(true)"
				>
					{{ formatMessage(messages.on) }}
				</Button>
				<Button
					size="sm"
					:type="booleanValue === false ? 'colored' : 'outlined'"
					:color="booleanValue === false ? 'brand' : undefined"
					:disabled="editorDisabled"
					:aria-pressed="booleanValue === false"
					@click="updateValue(false)"
				>
					{{ formatMessage(messages.off) }}
				</Button>
			</ButtonGroup>

			<select
				v-else-if="setting.editor.type === 'enum'"
				:value="valueText"
				:disabled="editorDisabled"
				:aria-label="formatMessage(messages.valueLabel)"
				class="h-9 w-48 rounded-xl border border-solid border-surface-5 bg-surface-4 px-3 text-sm font-medium text-primary outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow disabled:cursor-not-allowed disabled:opacity-50"
				@change="updateSelectValue"
			>
				<option v-if="!valueText" value="" disabled>{{ placeholder }}</option>
				<option v-for="choice in setting.editor.choices ?? []" :key="choice.value" :value="choice.value">
					{{ formatGameSettingChoice(formatMessage, setting.option_id, choice.value) }}
				</option>
			</select>

			<Input
				v-else
				:model-value="valueText"
				:size="'small'"
				:type="setting.editor.type === 'integer' || setting.editor.type === 'decimal' ? 'number' : 'text'"
				:min="inputMin"
				:max="inputMax"
				:step="inputStep"
				:placeholder="placeholder"
				:disabled="editorDisabled"
				:aria-label="formatMessage(messages.valueLabel)"
				wrapper-class="w-48"
				@update:model-value="updateValue"
			/>

			<span v-tooltip="syncToggleDisabled ? syncDisabledReason : undefined" class="flex">
				<Toggle
					:id="`sync-game-setting-${setting.option_id}`"
					:model-value="setting.sync_enabled"
					:disabled="syncToggleDisabled"
					:aria-label="formatMessage(messages.syncSetting, { setting: settingLabel })"
					@update:model-value="(enabled) => emit('update:sync-enabled', enabled)"
				/>
			</span>
		</div>
	</div>
</template>
