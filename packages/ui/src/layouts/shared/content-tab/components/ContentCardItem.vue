<script setup lang="ts">
import {
	ArrowLeftRightIcon,
	DownloadIcon,
	LockIcon,
	MoreVerticalIcon,
	SpinnerIcon,
	TrashExclamationIcon,
	TrashIcon,
	TriangleAlertIcon,
	UploadIcon,
} from '@modrinth/assets'
import { useMagicKeys } from '@vueuse/core'
import { computed, getCurrentInstance, ref } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import AutoLink from '#ui/components/base/AutoLink.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import BulletDivider from '#ui/components/base/BulletDivider.vue'
import type { ButtonMenuOption } from '#ui/components/base/buttons'
import { IconButton, TeleportOverflowMenu } from '#ui/components/base/buttons'
import Checkbox from '#ui/components/base/Checkbox.vue'
import ProgressSpinner from '#ui/components/base/ProgressSpinner.vue'
import Toggle from '#ui/components/base/Toggle.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'
import { truncatedTooltip } from '#ui/utils/truncate'

import type {
	ClientWarningType,
	ContentCardProject,
	ContentCardVersion,
	ContentOwner,
	ContentSource,
} from '../types'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	selectProject: {
		id: 'content.card.select-project',
		defaultMessage: 'Select {project}',
	},
	uploaded: {
		id: 'content.card.uploaded',
		defaultMessage: 'Uploaded',
	},
	frozen: {
		id: 'content.card.frozen',
		defaultMessage: 'This project is locked to its current version until unfrozen.',
	},
})

interface Props {
	project: ContentCardProject
	projectLink?: string | RouteLocationRaw
	version?: ContentCardVersion
	versionLink?: string | RouteLocationRaw
	owner?: ContentOwner
	source?: ContentSource
	external?: boolean
	enabled?: boolean
	locked?: boolean
	installing?: boolean
	installProgress?: number | null
	hasUpdate?: boolean
	isClientOnly?: boolean
	clientWarning?: ClientWarningType | null
	hideSwitchVersion?: boolean
	overflowOptions?: ButtonMenuOption[]
	disabled?: boolean
	disabledTooltip?: string | null
	toggleDisabled?: boolean
	toggleDisabledTooltip?: string | null
	hideToggle?: boolean
	showCheckbox?: boolean
	hideDelete?: boolean
	hideActions?: boolean
	inline?: boolean
}

const props = withDefaults(defineProps<Props>(), {
	projectLink: undefined,
	version: undefined,
	versionLink: undefined,
	owner: undefined,
	source: undefined,
	external: false,
	enabled: undefined,
	locked: false,
	installing: false,
	installProgress: undefined,
	hasUpdate: false,
	isClientOnly: false,
	clientWarning: null,
	hideSwitchVersion: false,
	overflowOptions: undefined,
	disabled: false,
	disabledTooltip: undefined,
	toggleDisabled: false,
	toggleDisabledTooltip: undefined,
	hideToggle: false,
	showCheckbox: false,
	hideDelete: false,
	hideActions: false,
	inline: false,
})

const selected = defineModel<boolean>('selected')

const emit = defineEmits<{
	'update:enabled': [value: boolean]
	select: [value: boolean, event?: MouseEvent]
	delete: [event: MouseEvent]
	update: []
	switchVersion: []
}>()

const instance = getCurrentInstance()
const hasDeleteListener = computed(() => typeof instance?.vnode.props?.onDelete === 'function')
const hasUpdateListener = computed(() => typeof instance?.vnode.props?.onUpdate === 'function')
const hasSwitchVersionListener = computed(
	() => typeof instance?.vnode.props?.onSwitchVersion === 'function',
)

const versionNumberRef = ref<HTMLElement | null>(null)
const fileNameRef = ref<HTMLElement | null>(null)

const isDisabled = computed(() => props.disabled || props.installing)
const isToggleDisabled = computed(() => isDisabled.value || props.toggleDisabled)

const clientWarningMessage = computed(() => {
	switch (props.clientWarning) {
		case 'retained':
			return commonMessages.clientRetainedWarning
		case 'depends':
			return commonMessages.clientDependsWarning
		default:
			return commonMessages.clientOnlyWarning
	}
})

const { shift: shiftHeld } = useMagicKeys()
const deleteHovered = ref(false)
const installTooltip = computed(() => {
	if (!props.installing) return undefined
	if (props.installProgress == null) return formatMessage(commonMessages.installingLabel)
	return `${formatMessage(commonMessages.installingLabel)} (${Math.round(props.installProgress)}%)`
})
</script>

<template>
	<div
		role="row"
		class="flex items-center justify-between"
		:class="{
			'h-[74px] gap-4 px-3': !inline,
			'gap-3': inline,
			'opacity-50 grayscale': disabled && !installing,
			'opacity-50': installing,
		}"
	>
		<div
			class="flex min-w-0 items-center gap-4"
			:class="
				hideActions ? 'flex-1' : 'flex-1 @[800px]:w-[45%] @[800px]:shrink-0 @[800px]:flex-none'
			"
		>
			<Checkbox
				v-if="showCheckbox"
				:model-value="selected ?? false"
				:aria-label="formatMessage(messages.selectProject, { project: project.title })"
				:disabled="isDisabled"
				class="shrink-0"
				@update:model-value="(value, event) => emit('select', value, event)"
			/>

			<div
				class="flex min-w-0 items-center gap-3 transition-[filter,opacity] duration-200"
				:class="enabled === false && !disabled ? 'grayscale opacity-50' : ''"
			>
				<div v-tooltip="installTooltip" class="relative flex shrink-0 items-center">
					<Avatar
						:src="project.icon_url"
						:alt="project.title"
						size="3rem"
						no-shadow
						class="rounded-2xl border border-surface-5"
					/>
					<div
						v-if="installing"
						class="absolute inset-0 flex items-center justify-center rounded-2xl bg-black/20"
					>
						<ProgressSpinner
							v-if="installProgress != null && installProgress > 0"
							:progress="installProgress"
							:max="100"
							class="size-5 text-white"
						/>
						<SpinnerIcon v-else class="size-5 animate-spin text-white" />
					</div>
				</div>
				<div class="flex min-w-0 flex-col gap-0.5">
					<div class="flex min-w-0 items-center gap-1">
						<AutoLink
							:target="
								typeof projectLink === 'string' && projectLink.startsWith('http')
									? '_blank'
									: undefined
							"
							:to="projectLink"
							class="truncate font-semibold leading-6 text-contrast !decoration-contrast"
							:class="{ 'hover:underline': projectLink }"
						>
							{{ project.title }}
						</AutoLink>
						<slot name="title-badges" />
						<span
							v-if="isClientOnly"
							v-tooltip="formatMessage(clientWarningMessage)"
							class="inline-flex size-5 shrink-0 cursor-help items-center justify-center"
							tabindex="0"
						>
							<TriangleAlertIcon class="pointer-events-none size-4 text-orange" />
						</span>
					</div>

					<div class="flex min-w-0 items-center gap-1">
						<template v-if="source">
							<AutoLink
								:target="
									typeof source.link === 'string' && source.link.startsWith('http')
										? '_blank'
										: undefined
								"
								:to="source.link"
								class="flex min-w-0 items-center gap-1 !decoration-secondary"
								:class="{ 'hover:underline': source.link }"
							>
								<Avatar
									:src="source.project.icon_url"
									:alt="source.project.title"
									:tint-by="source.project.id"
									size="1.25rem"
									no-shadow
									class="shrink-0 rounded-md"
								/>
								<span class="truncate text-sm leading-5 text-secondary">
									{{ source.project.title }}
								</span>
							</AutoLink>
						</template>
						<AutoLink
							v-else-if="owner"
							:target="
								typeof owner.link === 'string' && owner.link.startsWith('http')
									? '_blank'
									: undefined
							"
							:to="owner.link"
							class="flex shrink-0 items-center gap-1 !decoration-secondary"
							:class="{ 'hover:underline': owner.link }"
						>
							<Avatar
								:src="owner.avatar_url"
								:alt="owner.name"
								size="1.5rem"
								:circle="owner.type === 'user'"
								no-shadow
								class="shrink-0"
							/>
							<span class="text-sm leading-5 text-secondary">{{ owner.name }}</span>
						</AutoLink>
						<span v-else-if="external" class="flex items-center gap-1 text-secondary">
							<UploadIcon class="size-4 shrink-0" />
							<span class="text-sm leading-5">{{ formatMessage(messages.uploaded) }}</span>
						</span>
						<template v-if="version && !external">
							<BulletDivider class="shrink-0 @[800px]:hidden" />
							<AutoLink
								:target="
									typeof versionLink === 'string' && versionLink.startsWith('http')
										? '_blank'
										: undefined
								"
								:to="versionLink"
								class="truncate text-sm leading-5 text-secondary !decoration-secondary @[800px]:hidden"
								:class="{ 'hover:underline': versionLink }"
							>
								{{ version.version_number }}
							</AutoLink>
						</template>
					</div>
				</div>
			</div>
		</div>

		<div
			class="hidden flex-col gap-0.5 transition-[filter,opacity] duration-200 @[800px]:flex"
			:class="[
				hideActions ? 'flex-1' : 'flex-1 min-w-0',
				enabled === false && !disabled ? 'grayscale opacity-50' : '',
			]"
		>
			<template v-if="version">
				<AutoLink
					v-tooltip="truncatedTooltip(versionNumberRef, version.version_number)"
					:target="
						typeof versionLink === 'string' && versionLink.startsWith('http') ? '_blank' : undefined
					"
					:to="versionLink"
					class="inline-flex min-w-0 font-semibold leading-6 text-contrast !decoration-contrast"
					:class="{ 'hover:underline': versionLink, 'cursor-pointer': versionLink }"
				>
					<span ref="versionNumberRef" class="truncate">{{
						version.version_number.slice(0, Math.ceil(version.version_number.length / 2))
					}}</span
					><span class="shrink-0">{{
						version.version_number.slice(Math.ceil(version.version_number.length / 2))
					}}</span>
				</AutoLink>
				<span
					v-tooltip="truncatedTooltip(fileNameRef, version.file_name)"
					class="flex min-w-0 leading-6 text-secondary"
				>
					<span ref="fileNameRef" class="truncate">{{
						version.file_name.slice(0, Math.ceil(version.file_name.length / 2))
					}}</span
					><span class="shrink-0">{{
						version.file_name.slice(Math.ceil(version.file_name.length / 2))
					}}</span>
				</span>
			</template>
		</div>

		<div
			v-if="!hideActions"
			class="flex min-w-[160px] shrink-0 items-center justify-end gap-2 transition-colors duration-200"
		>
			<slot name="additionalButtonsLeft" />

			<!-- Fixed width container to reserve space for update/switch version button -->
			<div
				v-if="
					locked ||
					(hasUpdateListener && hasUpdate) ||
					(hasSwitchVersionListener && version && !hideSwitchVersion)
				"
				class="flex w-8 items-center justify-center"
			>
				<IconButton
					v-if="locked"
					v-tooltip="formatMessage(messages.frozen)"
					type="quiet"
					:label="formatMessage(messages.frozen)"
					disabled
				>
					<LockIcon class="size-5" />
				</IconButton>
				<IconButton
					v-else-if="hasUpdate"
					v-tooltip="
						isDisabled && disabledTooltip
							? disabledTooltip
							: formatMessage(commonMessages.updateAvailableLabel)
					"
					type="quiet"
					color="green"
					:label="
						isDisabled && disabledTooltip
							? disabledTooltip
							: formatMessage(commonMessages.updateAvailableLabel)
					"
					:disabled="isDisabled"
					class="hover:!bg-green focus-visible:!bg-green hover:!text-[var(--color-accent-contrast)] focus-visible:!text-[var(--color-accent-contrast)]"
					@click="emit('update')"
				>
					<DownloadIcon class="size-5" />
				</IconButton>
				<IconButton
					v-else-if="hasSwitchVersionListener && version && !hideSwitchVersion"
					v-tooltip="
						isDisabled && disabledTooltip
							? disabledTooltip
							: formatMessage(commonMessages.switchVersionButton)
					"
					type="quiet"
					:label="
						isDisabled && disabledTooltip
							? disabledTooltip
							: formatMessage(commonMessages.switchVersionButton)
					"
					:disabled="isDisabled"
					@click="emit('switchVersion')"
				>
					<ArrowLeftRightIcon class="size-5" />
				</IconButton>
			</div>

			<Toggle
				v-if="enabled !== undefined && !hideToggle"
				v-tooltip="
					isToggleDisabled && (toggleDisabledTooltip || disabledTooltip)
						? (toggleDisabledTooltip ?? disabledTooltip)
						: undefined
				"
				:model-value="enabled"
				:disabled="isToggleDisabled"
				:aria-label="project.title"
				class="my-auto"
				@update:model-value="(val) => emit('update:enabled', val as boolean)"
			/>

			<IconButton
				v-if="hasDeleteListener && !props.hideDelete"
				v-tooltip="
					isDisabled && disabledTooltip
						? disabledTooltip
						: formatMessage(
								shiftHeld && deleteHovered
									? commonMessages.deleteImmediatelyLabel
									: commonMessages.deleteLabel,
							)
				"
				type="quiet"
				:label="
					isDisabled && disabledTooltip
						? disabledTooltip
						: formatMessage(
								shiftHeld && deleteHovered
									? commonMessages.deleteImmediatelyLabel
									: commonMessages.deleteLabel,
							)
				"
				:disabled="isDisabled"
				@click="emit('delete', $event)"
				@mouseenter="deleteHovered = true"
				@mouseleave="deleteHovered = false"
			>
				<span class="relative size-5">
					<TrashIcon
						class="absolute inset-0 size-5 text-secondary transition-opacity duration-200"
						:class="shiftHeld && deleteHovered ? 'opacity-0' : 'opacity-100'"
					/>
					<TrashExclamationIcon
						class="absolute inset-0 size-5 text-red transition-opacity duration-200"
						:class="shiftHeld && deleteHovered ? 'opacity-100' : 'opacity-0'"
					/>
				</span>
			</IconButton>

			<slot name="additionalButtonsRight" />

			<TeleportOverflowMenu
				v-if="overflowOptions?.length"
				type="quiet"
				label="More options"
				:options="overflowOptions"
				:disabled="isDisabled"
			>
				<MoreVerticalIcon class="size-5" />
			</TeleportOverflowMenu>
		</div>
	</div>
</template>
