<script setup lang="ts">
import {
	BoxesIcon,
	ClockIcon,
	DownloadIcon,
	HeartIcon,
	MoreVerticalIcon,
	SpinnerIcon,
	TransferIcon,
} from '@modrinth/assets'
import { computed, getCurrentInstance } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import { useRelativeTime } from '../../composables/how-ago'
import { defineMessages, useVIntl } from '../../composables/i18n'
import { commonMessages } from '../../utils/common-messages'
import AutoLink from '../base/AutoLink.vue'
import Avatar from '../base/Avatar.vue'
import BulletDivider from '../base/BulletDivider.vue'
import ButtonStyled from '../base/ButtonStyled.vue'
import OverflowMenu, { type Option as OverflowMenuOption } from '../base/OverflowMenu.vue'
import TeleportOverflowMenu from '../servers/files/explorer/TeleportOverflowMenu.vue'
import type {
	ContentModpackCardCategory,
	ContentModpackCardProject,
	ContentModpackCardVersion,
	ContentOwner,
} from './types'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	updating: {
		id: 'content.modpack-card.updating',
		defaultMessage: 'Updating...',
	},
})

interface Props {
	project: ContentModpackCardProject
	projectLink?: string | RouteLocationRaw
	version?: ContentModpackCardVersion
	versionLink?: string | RouteLocationRaw
	owner?: ContentOwner
	categories?: ContentModpackCardCategory[]
	disabled?: boolean
	overflowOptions?: OverflowMenuOption[]
	hasUpdate?: boolean
	disabledText?: string
}

const props = withDefaults(defineProps<Props>(), {
	projectLink: undefined,
	version: undefined,
	versionLink: undefined,
	owner: undefined,
	categories: undefined,
	disabled: false,
	overflowOptions: undefined,
	hasUpdate: false,
	disabledText: undefined,
})

const emit = defineEmits<{
	update: []
	content: []
}>()

const instance = getCurrentInstance()
const hasUpdateListener = computed(() => typeof instance?.vnode.props?.onUpdate === 'function')
const hasContentListener = computed(() => typeof instance?.vnode.props?.onContent === 'function')

const formatTimeAgo = useRelativeTime()

const formatCompact = (n: number | undefined) => {
	if (n === undefined) return ''
	return new Intl.NumberFormat('en', { notation: 'compact', maximumFractionDigits: 2 }).format(n)
}

const collapsedOptions = computed(() => {
	const options: {
		id: string
		action: () => void
		color?: 'standard' | 'red' | 'brand' | 'orange' | 'green' | 'blue' | 'purple'
	}[] = []
	if (hasUpdateListener.value && !props.hasUpdate) {
		options.push({
			id: 'update',
			action: () => emit('update'),
		})
	}
	if (hasContentListener.value) {
		options.push({
			id: 'content',
			action: () => emit('content'),
		})
	}
	return options
})
</script>

<template>
	<div
		class="@container flex flex-col gap-4 rounded-[20px] bg-bg-raised p-6 shadow-md"
		:class="{ 'opacity-50': disabled }"
	>
		<div class="flex flex-wrap items-start justify-between gap-4">
			<div class="group flex min-w-0 flex-1 items-start gap-4">
				<AutoLink :to="projectLink" class="shrink-0">
					<Avatar :src="project.icon_url" :alt="project.title" size="5rem" no-shadow raised />
				</AutoLink>
				<div class="flex flex-col gap-1.5">
					<AutoLink
						:to="projectLink"
						class="text-xl font-semibold leading-8 text-contrast hover:underline group-hover:underline"
					>
						{{ project.title }}
					</AutoLink>
					<div class="flex flex-nowrap items-center gap-2 overflow-hidden text-secondary">
						<AutoLink
							v-if="owner"
							:to="owner.link"
							class="flex shrink-0 items-center gap-1.5 hover:underline"
						>
							<Avatar
								:src="owner.avatar_url"
								:alt="owner.name"
								size="2rem"
								:circle="owner.type === 'user'"
								no-shadow
							/>
							<span class="font-medium whitespace-nowrap">{{ owner.name }}</span>
						</AutoLink>
						<template v-if="version">
							<BulletDivider v-if="owner" />
							<AutoLink
								:to="versionLink"
								class="shrink-0 font-medium text-secondary !decoration-secondary whitespace-nowrap"
								:class="versionLink ? 'hover:underline' : ''"
							>
								v{{ version.version_number }}
							</AutoLink>
						</template>
						<template v-if="version?.date_published">
							<BulletDivider />
							<div class="flex shrink-0 items-center gap-2 whitespace-nowrap">
								<ClockIcon class="size-5" />
								<span>{{ formatTimeAgo(new Date(version.date_published)) }}</span>
							</div>
						</template>
					</div>
				</div>
			</div>

			<div class="flex shrink-0 items-center gap-2">
				<template v-if="disabled">
					<div class="flex items-center gap-2 text-secondary">
						<SpinnerIcon class="animate-spin" />
						<span class="font-semibold">{{
							disabledText ?? formatMessage(messages.updating)
						}}</span>
					</div>
				</template>
				<template v-else>
					<!-- Expanded actions visible at >= 700px -->
					<div class="hidden @[700px]:flex items-center gap-2">
						<ButtonStyled
							v-if="hasUpdateListener"
							:type="hasUpdate ? 'transparent' : 'outlined'"
							:color="hasUpdate ? 'green' : undefined"
							:color-fill="hasUpdate ? 'text' : undefined"
						>
							<button
								class="flex items-center gap-2"
								:class="[hasUpdate ? '' : '!border !border-surface-4']"
								@click="emit('update')"
							>
								<DownloadIcon v-if="hasUpdate" class="!text-green" />
								<TransferIcon v-else />
								<span class="font-semibold">{{
									formatMessage(
										hasUpdate ? commonMessages.updateButton : commonMessages.switchVersionButton,
									)
								}}</span>
							</button>
						</ButtonStyled>

						<ButtonStyled v-if="hasContentListener">
							<button class="!shadow-none" @click="emit('content')">
								<BoxesIcon />
								{{ formatMessage(commonMessages.contentLabel) }}
							</button>
						</ButtonStyled>
					</div>

					<!-- Collapsed actions visible at < 700px -->
					<div v-if="hasUpdate && hasUpdateListener" class="flex @[700px]:hidden">
						<ButtonStyled circular type="transparent" color="green" color-fill="text">
							<button
								v-tooltip="formatMessage(commonMessages.updateButton)"
								@click="emit('update')"
							>
								<DownloadIcon class="size-5" />
							</button>
						</ButtonStyled>
					</div>
					<ButtonStyled v-if="collapsedOptions.length" circular type="outlined"
						><TeleportOverflowMenu
							:options="collapsedOptions"
							class="flex @[700px]:hidden"
							btn-class="!border-surface-4 !border"
						>
							<MoreVerticalIcon class="size-5" />
							<template #update>
								<TransferIcon class="size-5" />
								{{ formatMessage(commonMessages.switchVersionButton) }}
							</template>
							<template #content>
								<BoxesIcon class="size-5" />
								{{ formatMessage(commonMessages.contentLabel) }}
							</template>
						</TeleportOverflowMenu></ButtonStyled
					>

					<ButtonStyled
						v-if="overflowOptions?.length"
						circular
						type="transparent"
						class="hidden @[700px]:flex"
					>
						<OverflowMenu :options="overflowOptions">
							<MoreVerticalIcon class="size-5" />
						</OverflowMenu>
					</ButtonStyled>
				</template>
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
				<div
					v-for="cat in categories"
					:key="cat.name"
					class="px-2 py-1 bg-surface-4 border border-solid rounded-full border-surface-5 text-secondary font-semibold"
					@click="cat.action"
				>
					{{ cat.name }}
				</div>
			</div>
		</div>
	</div>
</template>
