<script setup lang="ts">
import { RadioButtonCheckedIcon, RadioButtonIcon } from '@modrinth/assets'
import { Checkbox, defineMessages, useVIntl } from '@modrinth/ui'
import { computed, ref, useId } from 'vue'

export interface ProjectAccessPreset {
	id: string
	label: string
	description: string
	mask: number
}

export interface ProjectAccessGroup {
	id: string
	label: string
	options: { bit: number; label: string; destructive?: boolean }[]
}

const props = defineProps<{
	presets: ProjectAccessPreset[]
	groups: ProjectAccessGroup[]
	grantable: number
	disabled?: boolean
	title?: string
}>()

const model = defineModel<number>({ required: true })
const { formatMessage } = useVIntl()
const id = useId()
const custom = ref(false)
const customPermissions = ref(model.value)

const messages = defineMessages({
	access: { id: 'project.access-picker.access', defaultMessage: 'Access' },
	custom: { id: 'project.access-picker.custom', defaultMessage: 'Custom' },
	customDescription: {
		id: 'project.access-picker.custom.description',
		defaultMessage: 'Choose exactly which permissions they have.',
	},
	selected: {
		id: 'project.access-picker.custom.selected',
		defaultMessage: '{count, plural, other {# selected}}',
	},
})

const visibleMask = computed(() =>
	props.groups.flatMap((group) => group.options).reduce((mask, option) => mask | option.bit, 0),
)
const selectedPreset = computed(() =>
	custom.value
		? undefined
		: props.presets.find(({ mask }) => mask === (model.value & visibleMask.value)),
)
const selectedOption = computed(() => selectedPreset.value?.id ?? 'custom')
const accessOptions = computed(() => [
	...props.presets.map((preset) => ({ ...preset, disabled: !canApplyPreset(preset.mask) })),
	{
		id: 'custom',
		label: formatMessage(messages.custom),
		description: formatMessage(messages.customDescription),
		mask: selectedPreset.value ? customPermissions.value : model.value,
		disabled: false,
	},
])
const selectedCount = computed(
	() =>
		props.groups.flatMap((group) => group.options).filter(({ bit }) => !!(model.value & bit))
			.length,
)
const groups = computed(() =>
	props.groups.map((group) => {
		const mask = group.options.reduce((mask, option) => mask | option.bit, 0)
		return { ...group, mask, editable: mask & props.grantable }
	}),
)

function presetValue(mask: number) {
	return (model.value & ~visibleMask.value) | mask
}

function canApplyPreset(mask: number) {
	const next = presetValue(mask)
	return (next & props.grantable) === next && ((model.value ^ next) & ~props.grantable) === 0
}

function choosePreset(mask: number) {
	if (!selectedPreset.value) customPermissions.value = model.value
	custom.value = false
	model.value = presetValue(mask)
}

function chooseCustom() {
	if (!selectedPreset.value) return
	custom.value = true
	model.value = customPermissions.value
}

function chooseOption(option: ProjectAccessPreset) {
	if (option.id === 'custom') {
		chooseCustom()
	} else {
		choosePreset(option.mask)
	}
}

function setPermissions(mask: number, enabled: boolean) {
	if (selectedOption.value !== 'custom' || props.disabled) return
	custom.value = true
	model.value = enabled ? model.value | mask : model.value & ~mask
}
</script>

<template>
	<fieldset class="m-0 min-w-0 border-0 p-0">
		<legend class="mb-3 p-0 text-lg font-semibold text-contrast">
			{{ title ?? formatMessage(messages.access) }}
		</legend>
		<div class="overflow-hidden rounded-xl border border-solid border-surface-5">
			<div
				v-for="option in accessOptions"
				:key="option.id"
				class="border-0 border-t border-solid border-surface-5 first:border-t-0"
				:class="{ 'bg-surface-3': selectedOption === option.id }"
			>
				<label
					class="access-option relative flex items-start gap-3 p-4"
					:class="disabled || option.disabled ? 'cursor-not-allowed opacity-60' : 'cursor-pointer'"
				>
					<input
						type="radio"
						:name="id"
						:value="option.id"
						:checked="selectedOption === option.id"
						:disabled="disabled || option.disabled"
						:aria-labelledby="`${id}-${option.id}-title`"
						:aria-describedby="`${id}-${option.id}-description`"
						class="peer sr-only"
						@change="chooseOption(option)"
					/>
					<component
						:is="selectedOption === option.id ? RadioButtonCheckedIcon : RadioButtonIcon"
						aria-hidden="true"
						class="size-5 shrink-0 rounded-full peer-focus-visible:outline peer-focus-visible:outline-2 peer-focus-visible:outline-offset-2 peer-focus-visible:outline-brand"
						:class="selectedOption === option.id ? 'text-brand' : 'text-secondary'"
					/>
					<span class="flex min-w-0 flex-1 flex-col gap-1">
						<span class="flex flex-wrap items-baseline justify-between gap-x-2">
							<span
								:id="`${id}-${option.id}-title`"
								class="access-option-title font-semibold text-contrast"
							>
								{{ option.label }}
							</span>
							<span
								v-if="option.id === 'custom' && selectedOption === option.id"
								class="text-sm text-secondary"
							>
								{{ formatMessage(messages.selected, { count: selectedCount }) }}
							</span>
						</span>
						<span :id="`${id}-${option.id}-description`" class="text-sm text-secondary">
							{{ option.description }}
						</span>
					</span>
				</label>
				<Transition
					name="access-permissions"
					@before-leave="(panel) => panel.setAttribute('inert', '')"
				>
					<div v-if="selectedOption === option.id" class="access-permissions">
						<div class="min-h-0 overflow-hidden">
							<div class="grid gap-3 px-4 pb-4 sm:grid-cols-2 sm:pl-12">
								<div
									v-for="group in groups"
									:key="group.id"
									class="flex min-w-0 flex-col gap-3 rounded-xl border border-solid border-surface-5 p-3"
								>
									<Checkbox
										:label="group.label"
										label-class="font-semibold"
										class="disabled:opacity-60"
										:model-value="!!(option.mask & group.mask)"
										:indeterminate="
											!!(option.mask & group.mask) && (option.mask & group.mask) !== group.mask
										"
										:disabled="disabled || option.id !== 'custom' || !group.editable"
										@update:model-value="
											setPermissions(group.editable, (model & group.editable) !== group.editable)
										"
									/>
									<div
										class="flex flex-col gap-2 border-0 border-t border-solid border-surface-5 pt-3"
									>
										<Checkbox
											v-for="permission in group.options"
											:key="permission.bit"
											:label="permission.label"
											:label-class="permission.destructive ? 'text-red' : 'text-primary'"
											class="disabled:opacity-60"
											:model-value="!!(option.mask & permission.bit)"
											:disabled="
												disabled || option.id !== 'custom' || !(grantable & permission.bit)
											"
											@update:model-value="setPermissions(permission.bit, $event)"
										/>
									</div>
								</div>
							</div>
						</div>
					</div>
				</Transition>
			</div>
		</div>
	</fieldset>
</template>

<style scoped>
.access-option:has(input:enabled):is(:hover, :focus-within) .access-option-title {
	filter: brightness(var(--hover-brightness));
}

.access-permissions {
	display: grid;
	grid-template-rows: 1fr;
}

@media (prefers-reduced-motion: no-preference) {
	.access-permissions-enter-active,
	.access-permissions-leave-active {
		transition:
			grid-template-rows 180ms ease,
			opacity 180ms ease;
	}

	.access-permissions-enter-from,
	.access-permissions-leave-to {
		grid-template-rows: 0fr;
		opacity: 0;
		pointer-events: none;
	}
}
</style>
