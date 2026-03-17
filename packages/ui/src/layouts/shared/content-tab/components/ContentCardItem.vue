<script setup lang="ts">
import {
	DownloadIcon,
	MoreVerticalIcon,
	SpinnerIcon,
	TrashExclamationIcon,
	TrashIcon,
	TriangleAlertIcon,
} from '@modrinth/assets'
import { useMagicKeys } from '@vueuse/core'
import { Tooltip } from 'floating-vue'
import { computed, getCurrentInstance, ref } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import AutoLink from '#ui/components/base/AutoLink.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import BulletDivider from '#ui/components/base/BulletDivider.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import Checkbox from '#ui/components/base/Checkbox.vue'
import type { Option as OverflowMenuOption } from '#ui/components/base/OverflowMenu.vue'
import Toggle from '#ui/components/base/Toggle.vue'
import TeleportOverflowMenu from '#ui/components/servers/files/explorer/TeleportOverflowMenu.vue'
import { useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'
import { truncatedTooltip } from '#ui/utils/truncate'

import type { ContentCardProject, ContentCardVersion, ContentOwner } from '../types'

const { formatMessage } = useVIntl()

interface Props {
	project: ContentCardProject
	projectLink?: string | RouteLocationRaw
	version?: ContentCardVersion
	versionLink?: string | RouteLocationRaw
	owner?: ContentOwner
	enabled?: boolean
	installing?: boolean
	hasUpdate?: boolean
	isClientOnly?: boolean
	overflowOptions?: OverflowMenuOption[]
	disabled?: boolean
	showCheckbox?: boolean
	hideDelete?: boolean
	hideActions?: boolean
}

const props = withDefaults(defineProps<Props>(), {
	projectLink: undefined,
	version: undefined,
	versionLink: undefined,
	owner: undefined,
	enabled: undefined,
	installing: false,
	hasUpdate: false,
	isClientOnly: false,
	overflowOptions: undefined,
	disabled: false,
	showCheckbox: false,
	hideDelete: false,
	hideActions: false,
})

const selected = defineModel<boolean>('selected')

const emit = defineEmits<{
	'update:enabled': [value: boolean]
	delete: [event: MouseEvent]
	update: []
}>()

const instance = getCurrentInstance()
const hasDeleteListener = computed(() => typeof instance?.vnode.props?.onDelete === 'function')
const hasUpdateListener = computed(() => typeof instance?.vnode.props?.onUpdate === 'function')

const versionNumberRef = ref<HTMLElement | null>(null)
const fileNameRef = ref<HTMLElement | null>(null)

const { shift: shiftHeld } = useMagicKeys()
const deleteHovered = ref(false)
</script>

<template>
	<div
		role="row"
		class="flex h-[74px] items-center justify-between gap-4 px-3"
		:class="{ 'opacity-50': disabled }"
	>
		<div
			class="flex min-w-0 items-center gap-4 transition-[filter,opacity] duration-200"
			:class="[
				hideActions ? 'flex-1' : 'flex-1 @[800px]:w-[350px] @[800px]:shrink-0 @[800px]:flex-none',
				enabled === false && !disabled ? 'grayscale opacity-50' : '',
			]"
		>
			<Checkbox
				v-if="showCheckbox"
				:model-value="selected ?? false"
				:disabled="disabled"
				:aria-label="`Select ${project.title}`"
				class="shrink-0"
				@update:model-value="selected = $event"
			/>

			<div class="flex min-w-0 items-center gap-3">
				<div
					v-tooltip="installing ? formatMessage(commonMessages.installingLabel) : undefined"
					class="relative shrink-0"
				>
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
						<SpinnerIcon class="size-5 animate-spin text-white" />
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
						<Tooltip v-if="isClientOnly">
							<TriangleAlertIcon class="size-4 shrink-0 text-orange" />
							<template #popper>
								<div class="max-w-[18rem] text-sm">
									{{ formatMessage(commonMessages.clientOnlyWarning) }}
								</div>
							</template>
						</Tooltip>
					</div>

					<div class="flex min-w-0 items-center gap-1">
						<AutoLink
							v-if="owner"
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
						<template v-if="version">
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
					class="inline-flex min-w-0 font-medium leading-6 text-contrast !decoration-contrast"
					:class="{ 'hover:underline': versionLink, 'cursor-pointer': versionLink }"
				>
					<span ref="versionNumberRef" class="truncate">{{
						version.version_number.slice(0, Math.ceil(version.version_number.length / 2))
					}}</span>
					<span class="shrink-0">{{
						version.version_number.slice(Math.ceil(version.version_number.length / 2))
					}}</span>
				</AutoLink>
				<span
					v-tooltip="truncatedTooltip(fileNameRef, version.file_name)"
					class="flex min-w-0 leading-6 text-secondary"
				>
					<span ref="fileNameRef" class="truncate">{{
						version.file_name.slice(0, Math.ceil(version.file_name.length / 2))
					}}</span>
					<span class="shrink-0">{{
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

			<!-- Fixed width container to reserve space for update button -->
			<div v-if="hasUpdateListener" class="flex w-8 items-center justify-center">
				<ButtonStyled
					v-if="hasUpdate"
					circular
					type="transparent"
					color="green"
					color-fill="text"
					hover-color-fill="background"
				>
					<button
						v-tooltip="formatMessage(commonMessages.updateAvailableLabel)"
						:disabled="disabled"
						@click="emit('update')"
					>
						<DownloadIcon class="size-5" />
					</button>
				</ButtonStyled>
			</div>

			<Toggle
				v-if="enabled !== undefined"
				:model-value="enabled"
				:disabled="disabled"
				:aria-label="project.title"
				class="mr-2 my-auto"
				@update:model-value="(val) => emit('update:enabled', val as boolean)"
			/>

			<ButtonStyled v-if="hasDeleteListener && !props.hideDelete" circular type="transparent">
				<button
					v-tooltip="
						formatMessage(
							shiftHeld && deleteHovered
								? commonMessages.deleteImmediatelyLabel
								: commonMessages.deleteLabel,
						)
					"
					:disabled="disabled"
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
				</button>
			</ButtonStyled>

			<slot name="additionalButtonsRight" />

			<ButtonStyled circular type="transparent">
				<TeleportOverflowMenu
					v-if="overflowOptions?.length"
					:options="overflowOptions"
					:disabled="disabled"
				>
					<MoreVerticalIcon class="size-5" />
				</TeleportOverflowMenu>
			</ButtonStyled>
		</div>
	</div>
</template>
