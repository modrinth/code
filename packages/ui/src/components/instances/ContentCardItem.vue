<script setup lang="ts">
import { DownloadIcon, MoreVerticalIcon, OrganizationIcon, TrashIcon } from '@modrinth/assets'
import { computed, getCurrentInstance } from 'vue'

import Avatar from '../base/Avatar.vue'
import ButtonStyled from '../base/ButtonStyled.vue'
import Checkbox from '../base/Checkbox.vue'
import OverflowMenu, { type Option as OverflowMenuOption } from '../base/OverflowMenu.vue'
import Toggle from '../base/Toggle.vue'
import type { ContentCardProject, ContentCardVersion, ContentOwner } from './types'

interface Props {
	project: ContentCardProject
	version?: ContentCardVersion
	owner?: ContentOwner
	enabled?: boolean
	overflowOptions?: OverflowMenuOption[]
	disabled?: boolean
	showCheckbox?: boolean
}

withDefaults(defineProps<Props>(), {
	version: undefined,
	owner: undefined,
	enabled: undefined,
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
</script>

<template>
	<div
		class="flex items-center justify-between gap-4 px-4 py-3"
		:class="{ 'opacity-50': disabled }"
	>
		<!-- Checkbox + Project column -->
		<div class="flex min-w-0 shrink-0 items-center gap-4" :class="showCheckbox ? 'w-[350px]' : ''">
			<Checkbox
				v-if="showCheckbox"
				:model-value="selected ?? false"
				class="shrink-0"
				@update:model-value="selected = $event"
			/>

			<div class="flex min-w-0 items-center gap-3">
				<Avatar
					:src="project.icon_url"
					:alt="project.title"
					size="3.5rem"
					no-shadow
					class="shrink-0 rounded-2xl border border-surface-5"
				/>
				<div class="flex min-w-0 flex-col gap-1.5">
					<span class="truncate font-semibold leading-6 text-contrast">
						{{ project.title }}
					</span>

					<div v-if="owner" class="flex items-center gap-1">
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
					</div>
				</div>
			</div>
		</div>

		<!-- Version column -->
		<div class="hidden w-[335px] shrink-0 flex-col gap-1.5 md:flex">
			<template v-if="version">
				<span class="font-medium leading-6 text-contrast">{{ version.version_number }}</span>
				<span class="leading-6 text-secondary">{{ version.file_name }}</span>
			</template>
		</div>

		<!-- Actions column -->
		<div class="flex shrink-0 items-center gap-2">
			<slot name="additionalButtonsLeft" />

			<ButtonStyled
				v-if="hasUpdateListener"
				circular
				type="transparent"
				color="green"
				color-fill="text"
				hover-color-fill="background"
			>
				<button v-tooltip="'Update available'" @click="emit('update')">
					<DownloadIcon class="size-5" />
				</button>
			</ButtonStyled>

			<Toggle
				v-if="enabled !== undefined"
				:model-value="enabled"
				@update:model-value="(val) => emit('update:enabled', val as boolean)"
			/>

			<ButtonStyled v-if="hasDeleteListener" circular type="transparent">
				<button v-tooltip="'Delete'" @click="emit('delete')">
					<TrashIcon class="size-5 text-secondary" />
				</button>
			</ButtonStyled>

			<slot name="additionalButtonsRight" />

			<OverflowMenu v-if="overflowOptions?.length" :options="overflowOptions">
				<ButtonStyled circular type="transparent">
					<button>
						<MoreVerticalIcon class="size-5" />
					</button>
				</ButtonStyled>
			</OverflowMenu>
		</div>
	</div>
</template>
