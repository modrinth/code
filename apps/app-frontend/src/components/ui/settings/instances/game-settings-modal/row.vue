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

import GameSettingBooleanControl from './boolean-control.vue'
import {
	canonicalBooleanValue,
	canonicalValueFromInput,
	canonicalValueText,
	settingCanBeEnabled,
} from './editors'
import GameKeybindInput from './keybind-input.vue'
import {
	formatGameSettingChoice,
	formatGameSettingDescription,
	formatGameSettingLabel,
	formatGameSettingValidation,
	presentationMessages,
} from './messages'

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
	syncSetting: {
		id: 'app.settings.synced-options.game-settings.sync-setting',
		defaultMessage: 'Sync {setting} across instances',
	},
	unsyncSetting: {
		id: 'app.settings.synced-options.game-settings.unsync-setting',
		defaultMessage: 'Stop syncing {setting} across instances',
	},
	syncedAcrossInstances: {
		id: 'app.settings.synced-options.game-settings.synced-across-instances',
		defaultMessage: 'Synced across instances',
	},
	notBeingSynced: {
		id: 'app.settings.synced-options.game-settings.not-being-synced',
		defaultMessage: 'Not being synced',
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
const isNumber = computed(
	() => props.setting.editor.type === 'integer' || props.setting.editor.type === 'decimal',
)
const isSlider = computed(
	() => isNumber.value && props.setting.editor.min != null && props.setting.editor.max != null,
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
	if (valueText.value === '') return null
	const value = Number(valueText.value)
	return Number.isFinite(value) ? value : null
})
const inputValue = computed(() =>
	isNumber.value ? (sliderValue.value ?? undefined) : valueText.value,
)
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
		(!props.setting.sync_enabled && !settingCanBeEnabled(props.setting)),
)
const placeholder = computed(() => {
	if (props.setting.value_state === 'mixed') return formatMessage(messages.mixed)
	if (props.setting.value_state === 'unset') return formatMessage(messages.unset)
	if (props.setting.editor.type === 'external_raw') {
		return formatMessage(presentationMessages.customValuePlaceholder)
	}
	return undefined
})
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
const syncActionLabel = computed(() =>
	formatMessage(props.setting.sync_enabled ? messages.unsyncSetting : messages.syncSetting, {
		setting: settingLabel.value,
	}),
)
const syncStatusTooltip = computed(() =>
	formatMessage(
		props.setting.sync_enabled ? messages.syncedAcrossInstances : messages.notBeingSynced,
	),
)

function updateValue(value: string | number | boolean | undefined) {
	emit('update:canonical-value', canonicalValueFromInput(props.setting, value))
}
</script>

<template>
	<div
		class="grid min-h-[54px] min-w-0 items-center gap-2"
		:class="
			isSlider
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
					v-if="validationMessage"
					v-tooltip="validationMessage"
					tabindex="0"
					role="img"
					:aria-label="validationMessage"
					class="flex shrink-0 rounded-md text-orange outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow"
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
				v-if="isSlider"
				:model-value="sliderValue"
				:min="inputMin ?? 0"
				:max="inputMax ?? 100"
				:step="inputStep ?? 1"
				:snap-points="setting.category_id === 'music_and_sound' ? [0, 50, 100] : []"
				:snap-range="5"
				:unit="setting.editor.unit === 'percent' ? '%' : undefined"
				:placeholder="placeholder"
				:aria-label="settingLabel"
				:disabled="editorDisabled"
				@update:model-value="updateValue"
			/>

			<GameSettingBooleanControl
				v-else-if="setting.editor.type === 'bool'"
				:model-value="booleanValue"
				:label="settingLabel"
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
				:aria-label="settingLabel"
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
				:model-value="inputValue"
				:type="isNumber ? 'number' : 'text'"
				:min="inputMin"
				:max="inputMax"
				:step="inputStep"
				:placeholder="placeholder"
				:disabled="editorDisabled"
				:aria-label="settingLabel"
				wrapper-class="w-full"
				@update:model-value="updateValue"
			/>
		</div>

		<span
			v-if="showSyncToggle"
			v-tooltip="syncToggleDisabled && syncDisabledReason ? syncDisabledReason : syncStatusTooltip"
			class="flex justify-center"
		>
			<IconButton
				:type="setting.sync_enabled ? 'outlined' : 'base'"
				:color="setting.sync_enabled ? 'blue' : undefined"
				:class="setting.sync_enabled ? '!bg-highlight-blue' : undefined"
				:label="syncActionLabel"
				:disabled="syncToggleDisabled"
				:aria-pressed="setting.sync_enabled"
				@click="emit('update:sync-enabled', !setting.sync_enabled)"
			>
				<LinkIcon v-if="setting.sync_enabled" aria-hidden="true" />
				<UnlinkIcon v-else aria-hidden="true" />
			</IconButton>
		</span>
	</div>
</template>
