<script setup lang="ts">
import {
	ClockIcon,
	DownloadIcon,
	HeartIcon,
	MoreVerticalIcon,
	OrganizationIcon,
	UnlinkIcon,
} from '@modrinth/assets'
import { computed, getCurrentInstance } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import { useRelativeTime } from '../../composables/how-ago'
import AutoLink from '../base/AutoLink.vue'
import Avatar from '../base/Avatar.vue'
import BulletDivider from '../base/BulletDivider.vue'
import ButtonStyled from '../base/ButtonStyled.vue'
import OverflowMenu, { type Option as OverflowMenuOption } from '../base/OverflowMenu.vue'
import TagItem from '../base/TagItem.vue'
import type {
	ContentModpackCardCategory,
	ContentModpackCardProject,
	ContentModpackCardVersion,
	ContentOwner,
} from './types'

interface Props {
	project: ContentModpackCardProject
	projectLink?: string | RouteLocationRaw
	version?: ContentModpackCardVersion
	owner?: ContentOwner
	categories?: ContentModpackCardCategory[]
	disabled?: boolean
	overflowOptions?: OverflowMenuOption[]
}

withDefaults(defineProps<Props>(), {
	projectLink: undefined,
	version: undefined,
	owner: undefined,
	categories: undefined,
	disabled: false,
	overflowOptions: undefined,
})

const emit = defineEmits<{
	update: []
	content: []
	unlink: []
}>()

const instance = getCurrentInstance()
const hasUpdateListener = computed(() => typeof instance?.vnode.props?.onUpdate === 'function')
const hasContentListener = computed(() => typeof instance?.vnode.props?.onContent === 'function')
const hasUnlinkListener = computed(() => typeof instance?.vnode.props?.onUnlink === 'function')

const formatTimeAgo = useRelativeTime()

const formatCompact = (n: number | undefined) => {
	if (n === undefined) return ''
	return new Intl.NumberFormat('en', { notation: 'compact', maximumFractionDigits: 2 }).format(n)
}
</script>

<template>
	<div
		class="flex flex-col gap-4 rounded-[20px] bg-bg-raised p-6 shadow-md"
		:class="{ 'opacity-50': disabled }"
	>
		<div class="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
			<div class="flex flex-col gap-4 sm:flex-row sm:items-center">
				<Avatar
					:src="project.icon_url"
					:alt="project.title"
					size="5rem"
					no-shadow
					raised
					class="shrink-0"
				/>
				<div class="flex flex-col gap-1.5">
					<AutoLink
						:to="projectLink"
						class="text-2xl font-semibold leading-8 text-contrast hover:underline"
					>
						{{ project.title }}
					</AutoLink>
					<div class="flex flex-wrap items-center gap-2 text-secondary">
						<template v-if="owner">
							<AutoLink :to="owner.link" class="flex items-center gap-1.5 hover:underline">
								<Avatar
									:src="owner.avatar_url"
									:alt="owner.name"
									size="2rem"
									:circle="owner.type === 'user'"
									no-shadow
								/>
								<OrganizationIcon v-if="owner.type === 'organization'" class="size-4" />
								<span class="font-medium">{{ owner.name }}</span>
							</AutoLink>
						</template>
						<template v-if="owner && version">
							<BulletDivider />
						</template>
						<template v-if="version">
							<span class="font-medium">v{{ version.version_number }}</span>
						</template>
						<template v-if="version?.date_published">
							<BulletDivider />
							<div class="flex items-center gap-2">
								<ClockIcon class="size-5" />
								<span>{{ formatTimeAgo(new Date(version.date_published)) }}</span>
							</div>
						</template>
					</div>
				</div>
			</div>

			<div class="flex shrink-0 items-center gap-2">
				<ButtonStyled v-if="hasUpdateListener" type="transparent" color="green" color-fill="text">
					<button class="flex items-center gap-2" @click="emit('update')">
						<DownloadIcon class="!text-green size-5" />
						<span class="font-semibold">Update</span>
					</button>
				</ButtonStyled>

				<ButtonStyled v-if="hasContentListener">
					<button class="!shadow-none" @click="emit('content')">Content</button>
				</ButtonStyled>

				<ButtonStyled v-if="hasUnlinkListener" circular type="outlined">
					<button
						v-tooltip="'Unlink modpack'"
						class="!border-surface-4 !border-[1px]"
						@click="emit('unlink')"
					>
						<UnlinkIcon class="size-5" />
					</button>
				</ButtonStyled>

				<ButtonStyled v-if="overflowOptions?.length" circular type="transparent">
					<OverflowMenu :options="overflowOptions">
						<MoreVerticalIcon class="size-5" />
					</OverflowMenu>
				</ButtonStyled>
			</div>
		</div>

		<span v-if="project.description" class="text-secondary">
			{{ project.description }}
		</span>

		<div class="flex flex-wrap items-center gap-3">
			<div v-if="project.downloads !== undefined" class="flex items-center gap-2 text-secondary">
				<DownloadIcon class="size-5" />
				<span class="font-medium">{{ formatCompact(project.downloads) }}</span>
			</div>

			<div v-if="project.followers !== undefined" class="flex items-center gap-2 text-secondary">
				<HeartIcon class="size-5" />
				<span class="font-medium">{{ formatCompact(project.followers) }}</span>
			</div>

			<div v-if="categories?.length" class="flex flex-wrap gap-2">
				<TagItem v-for="cat in categories" :key="cat.name" :action="cat.action">
					{{ cat.name }}
				</TagItem>
			</div>
		</div>
	</div>
</template>
