<script setup lang="ts">
import { DownloadIcon, MoreVerticalIcon, OrganizationIcon, TrashIcon } from '@modrinth/assets'
import { type ComponentPublicInstance, computed, getCurrentInstance, ref } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import { useVIntl } from '../../composables/i18n'
import { commonMessages } from '../../utils/common-messages'
import { truncatedTooltip } from '../../utils/truncate'
import AutoLink from '../base/AutoLink.vue'
import Avatar from '../base/Avatar.vue'
import ButtonStyled from '../base/ButtonStyled.vue'
import Checkbox from '../base/Checkbox.vue'
import type { Option as OverflowMenuOption } from '../base/OverflowMenu.vue'
import Toggle from '../base/Toggle.vue'
import TeleportOverflowMenu from '../servers/files/explorer/TeleportOverflowMenu.vue'
import type { ContentCardProject, ContentCardVersion, ContentOwner } from './types'

const { formatMessage } = useVIntl()

interface Props {
	project: ContentCardProject
	projectLink?: string | RouteLocationRaw
	version?: ContentCardVersion
	owner?: ContentOwner
	enabled?: boolean
	hasUpdate?: boolean
	overflowOptions?: OverflowMenuOption[]
	disabled?: boolean
	showCheckbox?: boolean
}

withDefaults(defineProps<Props>(), {
	projectLink: undefined,
	version: undefined,
	owner: undefined,
	enabled: undefined,
	hasUpdate: false,
	overflowOptions: undefined,
	disabled: false,
	showCheckbox: false,
})

const selected = defineModel<boolean>('selected')

const emit = defineEmits<{
	'update:enabled': [value: boolean]
	delete: []
	update: []
}>()

const instance = getCurrentInstance()
const hasDeleteListener = computed(() => typeof instance?.vnode.props?.onDelete === 'function')
const hasUpdateListener = computed(() => typeof instance?.vnode.props?.onUpdate === 'function')

const titleRef = ref<ComponentPublicInstance | null>(null)

const MAX_FILENAME_LENGTH = 42

function truncateMiddle(str: string, maxLength: number): string {
	if (str.length <= maxLength) return str
	const ellipsis = '...'
	const charsToShow = maxLength - ellipsis.length
	const frontChars = Math.ceil(charsToShow / 2)
	const backChars = Math.floor(charsToShow / 2)
	return str.slice(0, frontChars) + ellipsis + str.slice(-backChars)
}
</script>

<template>
	<div
		class="grid h-[74px] items-center gap-4 px-4"
		:class="[
			{ 'opacity-50': disabled },
			showCheckbox
				? 'grid-cols-[auto_1fr_1fr] md:grid-cols-[auto_1fr_335px_1fr]'
				: 'grid-cols-[1fr_1fr] md:grid-cols-[1fr_335px_1fr]',
		]"
	>
		<Checkbox
			v-if="showCheckbox"
			:model-value="selected ?? false"
			:disabled="disabled"
			class="shrink-0"
			@update:model-value="selected = $event"
		/>

		<div class="flex min-w-0 items-center gap-3">
			<Avatar
				:src="project.icon_url"
				:alt="project.title"
				size="3rem"
				no-shadow
				class="shrink-0 rounded-2xl border border-surface-5"
			/>
			<div class="flex min-w-0 flex-col gap-0.5">
				<AutoLink
					ref="titleRef"
					v-tooltip="truncatedTooltip(titleRef?.$el, project.title)"
					:target="
						typeof projectLink === 'string' && projectLink.startsWith('http') ? '_blank' : undefined
					"
					:to="projectLink"
					class="truncate font-semibold leading-6 text-contrast !decoration-contrast"
					:class="{ 'hover:underline': projectLink }"
				>
					{{ project.title }}
				</AutoLink>

				<AutoLink
					v-if="owner"
					:target="
						typeof owner.link === 'string' && owner.link.startsWith('http') ? '_blank' : undefined
					"
					:to="owner.link"
					class="flex items-center gap-1 !decoration-secondary"
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
					<OrganizationIcon v-if="owner.type === 'organization'" class="size-4 text-secondary" />
					<span class="text-sm leading-5 text-secondary">{{ owner.name }}</span>
				</AutoLink>
			</div>
		</div>

		<div class="hidden flex-col justify-center gap-0.5 md:flex">
			<template v-if="version">
				<span class="font-medium leading-6 text-contrast">{{ version.version_number }}</span>
				<span
					v-tooltip="version.file_name.length > MAX_FILENAME_LENGTH ? version.file_name : undefined"
					class="leading-6 text-secondary"
				>
					{{ truncateMiddle(version.file_name, MAX_FILENAME_LENGTH) }}
				</span>
			</template>
		</div>

		<div class="flex items-center justify-end gap-2">
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
				small
				@update:model-value="(val) => emit('update:enabled', val as boolean)"
			/>

			<ButtonStyled v-if="hasDeleteListener" circular type="transparent">
				<button
					v-tooltip="formatMessage(commonMessages.deleteLabel)"
					:disabled="disabled"
					@click="emit('delete')"
				>
					<TrashIcon class="size-5 text-secondary" />
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
