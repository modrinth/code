<script setup lang="ts">
import {
	BoxesIcon,
	ClockIcon,
	DownloadIcon,
	HeartIcon,
	MoreVerticalIcon,
	OrganizationIcon,
	TransferIcon,
	UnlinkIcon,
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
import type {
	ContentModpackCardCategory,
	ContentModpackCardProject,
	ContentModpackCardVersion,
	ContentOwner,
} from './types'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	unlinkModpack: {
		id: 'instances.modpack-card.unlink',
		defaultMessage: 'Unlink modpack',
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
}

withDefaults(defineProps<Props>(), {
	projectLink: undefined,
	version: undefined,
	versionLink: undefined,
	owner: undefined,
	categories: undefined,
	disabled: false,
	overflowOptions: undefined,
	hasUpdate: false,
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
						class="text-xl font-semibold leading-8 text-contrast hover:underline"
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
							<AutoLink
								:to="versionLink"
								class="font-medium text-secondary !decoration-secondary"
								:class="versionLink ? 'hover:underline' : ''"
							>
								v{{ version.version_number }}
							</AutoLink>
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

				<ButtonStyled v-if="hasUnlinkListener" circular type="outlined">
					<button
						v-tooltip="formatMessage(messages.unlinkModpack)"
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
