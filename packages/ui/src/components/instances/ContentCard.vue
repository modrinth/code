<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon, MoreVerticalIcon, OrganizationIcon, TrashIcon } from '@modrinth/assets'
import { computed, getCurrentInstance } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import Avatar from '../base/Avatar.vue'
import ButtonStyled from '../base/ButtonStyled.vue'
import OverflowMenu, { type Option as OverflowMenuOption } from '../base/OverflowMenu.vue'
import Toggle from '../base/Toggle.vue'

export type ContentCardProject = Pick<
	Labrinth.Projects.v2.Project,
	'id' | 'slug' | 'title' | 'icon_url'
>

export type ContentCardVersion = Pick<Labrinth.Versions.v2.Version, 'id' | 'version_number'> & {
	file_name: string
}

export interface ContentCardOwner {
	id: string
	name: string
	avatar_url?: string
	type: 'user' | 'organization'
	link?: string | RouteLocationRaw
}

interface Props {
	project: ContentCardProject
	version?: ContentCardVersion
	owner?: ContentCardOwner
	enabled?: boolean
	overflowOptions?: OverflowMenuOption[]
	disabled?: boolean
}

withDefaults(defineProps<Props>(), {
	version: undefined,
	owner: undefined,
	enabled: undefined,
	overflowOptions: undefined,
	disabled: false,
})

const emit = defineEmits<{
	'update:enabled': [value: boolean]
	delete: []
	update: []
}>()

const instance = getCurrentInstance()
const hasDeleteListener = computed(() => !!instance?.vnode.props?.onDelete)
const hasUpdateListener = computed(() => !!instance?.vnode.props?.onUpdate)
</script>

<template>
	<div
		class="grid grid-cols-[1fr_auto] items-center gap-4 rounded-[20px] bg-bg-raised p-4 shadow-md md:grid-cols-3"
		:class="{ 'opacity-50': disabled }"
	>
		<div class="flex min-w-0 items-center gap-4">
			<Avatar :src="project.icon_url" :alt="project.title" size="4rem" no-shadow raised />
			<div class="flex min-w-0 flex-col gap-1.5">
				<span class="truncate font-semibold text-contrast">
					{{ project.title }}
				</span>

				<div v-if="owner" class="flex items-center gap-1">
					<Avatar
						:src="owner.avatar_url"
						:alt="owner.name"
						size="1.5rem"
						:circle="owner.type === 'user'"
						no-shadow
					/>
					<OrganizationIcon v-if="owner.type === 'organization'" class="size-4 text-secondary" />
					<span class="text-sm text-secondary">{{ owner.name }}</span>
				</div>
			</div>
		</div>

		<div class="hidden flex-col gap-1.5 md:flex">
			<template v-if="version">
				<span class="font-medium text-contrast">{{ version.version_number }}</span>
				<span class="text-secondary">{{ version.file_name }}</span>
			</template>
		</div>

		<div class="flex items-center justify-self-end gap-2">
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
				@update:model-value="(val) => emit('update:enabled', val)"
			/>

			<ButtonStyled v-if="hasDeleteListener" circular type="transparent">
				<button v-tooltip="'Delete'" @click="emit('delete')">
					<TrashIcon class="size-5 text-secondary" />
				</button>
			</ButtonStyled>

			<slot name="additionalButtonsRight" />

			<ButtonStyled circular type="transparent">
				<OverflowMenu v-if="overflowOptions?.length" :options="overflowOptions">
					<MoreVerticalIcon class="size-5" />
				</OverflowMenu>
			</ButtonStyled>
		</div>
	</div>
</template>
