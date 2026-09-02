<script setup lang="ts">
import { LinkIcon, UnknownIcon, UnlinkIcon } from '@modrinth/assets'
import {
	Combobox,
	type ComboboxOption,
	defineMessages,
	IconButton,
	Input,
	Slider,
	truncatedTooltip,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import type { EditableGameSetting, GameOptionCanonicalValue } from '@/helpers/game-options'

import GameSettingBooleanControl from './game-setting-boolean-control.vue'
import {
	canonicalBooleanValue,
	canonicalValueFromInput,
	canonicalValueText,
} from './game-setting-editors'
import {
	formatCompatibilityTooltip,
	formatGameSettingChoice,
	formatGameSettingDescription,
	formatGameSettingLabel,
	formatGameSettingValidation,
	presentationMessages,
	shouldShowCompatibilityIndicator,
} from './game-setting-messages'
import GameKeybindInput from './GameKeybindInput.vue'

const props = withDefaults(
	defineProps<{
		setting: EditableGameSetting
		keybindConflicts?: string[]
		disabled?: boolean
		showSyncToggle?: boolean
	}>(),
	{
		keybindConflicts: () => [],
		disabled: false,
		showSyncToggle: true,
	},
)

const emit = defineEmits<{
	'update:sync-enabled': [enabled: boolean]
	'update:canonical-value': [value: GameOptionCanonicalValue | null]
}>()

const { formatMessage } = useVIntl()
const settingLabelRef = ref<HTMLElement | null>(null)

const messages = defineMessages({
	valueLabel: {
		id: 'app.settings.synced-options.game-settings.value-label',
		defaultMessage: 'Value',
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
		defaultMessage: 'Different across instances',
	},
	unset: {
		id: 'app.settings.synced-options.game-settings.value-unset',
		defaultMessage: 'Choose a value',
	},
	compatibilityDetails: {
		id: 'app.settings.synced-options.game-settings.compatibility-details',
		defaultMessage: 'Why this setting may not sync',
	},
	syncSetting: {
		id: 'app.settings.synced-options.game-settings.sync-setting',
		defaultMessage: 'Sync {setting} across instances',
	},
	unsyncSetting: {
		id: 'app.settings.synced-options.game-settings.unsync-setting',
		defaultMessage: 'Stop syncing {setting} across instances',
	},
})

const settingLabel = computed(() => formatGameSettingLabel(formatMessage, props.setting))
const settingDescription = computed(() =>
	formatGameSettingDescription(formatMessage, props.setting),
)
const valueText = computed(() => canonicalValueText(props.setting))
const enumOptions = computed<ComboboxOption<string>[]>(() =>
	(props.setting.editor.choices ?? []).map((choice) => ({
		value: choice.value,
		label: formatGameSettingChoice(formatMessage, props.setting.option_id, choice.value),
	})),
)
const isVolumeSlider = computed(
	() =>
		props.setting.category_id === 'music_and_sound' &&
		props.setting.editor.unit === 'percent' &&
		props.setting.editor.type === 'decimal',
)
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
const sliderValue = computed(() => {
	const value = Number(valueText.value)
	return Number.isFinite(value) ? value : (inputMin.value ?? 0)
})
const editorDisabled = computed(
	() =>
		props.disabled ||
		props.setting.controlled ||
		(props.showSyncToggle && !props.setting.sync_enabled),
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
const compatibilityTooltip = computed(() =>
	formatCompatibilityTooltip(formatMessage, props.setting),
)
const validationMessage = computed(() =>
	formatGameSettingValidation(formatMessage, props.setting.validation_error),
)
const detailsTooltip = computed(() => validationMessage.value ?? compatibilityTooltip.value)
const compatibilityTone = computed(() =>
	validationMessage.value || props.setting.compatibility.left_local > 0
		? 'text-orange'
		: 'text-secondary',
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
const syncActionLabel = computed(() =>
	formatMessage(props.setting.sync_enabled ? messages.unsyncSetting : messages.syncSetting, {
		setting: settingLabel.value,
	}),
)

function updateValue(value: string | number | boolean | undefined) {
	emit('update:canonical-value', canonicalValueFromInput(props.setting, value))
}
</script>

<template>
	<div
		class="grid min-h-[54px] min-w-0 items-center gap-2"
		:class="
			isVolumeSlider
				? showSyncToggle
					? 'grid-cols-[minmax(10rem,0.65fr)_minmax(0,1.35fr)_2.25rem]'
					: 'grid-cols-[minmax(10rem,0.65fr)_minmax(0,1.35fr)]'
				: showSyncToggle
					? 'grid-cols-[minmax(0,1fr)_12rem_2.25rem]'
					: 'grid-cols-[minmax(0,1fr)_12rem]'
		"
	>
		<div class="min-w-0 pr-2">
			<div class="flex items-center gap-2">
				<h3
					ref="settingLabelRef"
					v-tooltip="truncatedTooltip(settingLabelRef, settingLabel)"
					class="m-0 truncate text-lg font-semibold text-contrast"
				>
					{{ settingLabel }}
				</h3>
				<span
					v-if="validationMessage || shouldShowCompatibilityIndicator(setting)"
					v-tooltip="detailsTooltip"
					tabindex="0"
					role="img"
					:aria-label="formatMessage(messages.compatibilityDetails)"
					class="flex shrink-0 rounded-md outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow"
					:class="compatibilityTone"
				>
					<UnknownIcon class="size-4" aria-hidden="true" />
				</span>
			</div>
			<p v-if="settingDescription" class="m-0 mt-0.5 text-primary">
				{{ settingDescription }}
			</p>
		</div>

		<div class="flex min-w-0 items-center justify-end">
			<Slider
				v-if="isVolumeSlider"
				:model-value="sliderValue"
				:min="inputMin ?? 0"
				:max="inputMax ?? 100"
				:step="inputStep ?? 1"
				:snap-points="[0, 50, 100]"
				:snap-range="5"
				:disabled="editorDisabled"
				value-input-appearance="transparent"
				class="[&_input]:!text-secondary"
				@update:model-value="updateValue"
			/>

			<GameSettingBooleanControl
				v-else-if="setting.editor.type === 'bool'"
				:model-value="booleanValue"
				:label="formatMessage(messages.valueLabel)"
				:on-label="formatMessage(messages.on)"
				:off-label="formatMessage(messages.off)"
				:placeholder="placeholder"
				:disabled="editorDisabled"
				@update:model-value="updateValue"
			/>

			<Combobox
				v-else-if="setting.editor.type === 'enum'"
				:model-value="valueText"
				:options="enumOptions"
				:placeholder="placeholder"
				:disabled="editorDisabled"
				:aria-label="formatMessage(messages.valueLabel)"
				trigger-class="!bg-transparent !text-secondary"
				class="min-w-0"
				@update:model-value="updateValue"
			/>

			<GameKeybindInput
				v-else-if="setting.editor.type === 'key_binding'"
				:model-value="valueText"
				:setting-label="settingLabel"
				:conflicts="keybindConflicts"
				:mixed="setting.value_state === 'mixed'"
				:disabled="editorDisabled"
				@update:model-value="updateValue"
			/>

			<Input
				v-else
				:model-value="valueText"
				:type="
					setting.editor.type === 'integer' || setting.editor.type === 'decimal' ? 'number' : 'text'
				"
				:min="inputMin"
				:max="inputMax"
				:step="inputStep"
				:placeholder="placeholder"
				:disabled="editorDisabled"
				:aria-label="formatMessage(messages.valueLabel)"
				wrapper-class="w-full !bg-transparent"
				input-class="!text-secondary"
				@update:model-value="updateValue"
			/>
		</div>

		<span
			v-if="showSyncToggle"
			v-tooltip="syncToggleDisabled ? syncDisabledReason : undefined"
			class="flex justify-center"
		>
			<IconButton
				:label="syncActionLabel"
				:disabled="syncToggleDisabled"
				:aria-pressed="setting.sync_enabled"
				:class="
					setting.sync_enabled
						? '!bg-highlight-green !text-green !shadow-[inset_0_0_0_1px_var(--color-green)] [&>svg]:!text-green'
						: ''
				"
				@click="emit('update:sync-enabled', !setting.sync_enabled)"
			>
				<LinkIcon v-if="setting.sync_enabled" />
				<UnlinkIcon v-else />
			</IconButton>
		</span>
	</div>
</template>
