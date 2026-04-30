<template>
	<article
		class="flex min-h-[19.75rem] w-full flex-col overflow-hidden rounded-2xl border border-solid border-surface-5 bg-bg-raised shadow-xl"
		:class="(world as any)?.active ? '!border-brand' : ''"
	>
		<template v-if="world.type === 'empty'">
			<div class="flex flex-1 flex-col items-center pt-[3.125rem] text-center">
				<svg
					class="size-[5.6645rem]"
					viewBox="0 0 91 91"
					fill="none"
					xmlns="http://www.w3.org/2000/svg"
					aria-hidden="true"
				>
					<rect
						x="22.4356"
						y="0.629395"
						width="71"
						height="71"
						rx="19.5"
						transform="rotate(17.8856 22.4356 0.629395)"
						stroke="#42444A"
					/>
					<rect
						x="4.36354"
						y="16.5661"
						width="71"
						height="71"
						rx="19.5"
						transform="rotate(-8.79487 4.36354 16.5661)"
						fill="#34363C"
					/>
					<rect
						x="4.36354"
						y="16.5661"
						width="71"
						height="71"
						rx="19.5"
						transform="rotate(-8.79487 4.36354 16.5661)"
						stroke="#42444A"
					/>
					<g :clip-path="`url(#${emptyCardClipId})`">
						<path
							d="M47.4227 62.6919C56.5193 61.2846 62.7525 52.7695 61.3452 43.673C59.9378 34.5764 51.4227 28.3432 42.3262 29.7505C33.2297 31.1579 26.9964 39.673 28.4038 48.7695C29.8111 57.866 38.3262 64.0993 47.4227 62.6919Z"
							stroke="#B0BAC5"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
						/>
						<path
							d="M42.3246 29.7502C38.7824 34.8453 37.3358 41.1078 38.2846 47.2402C39.2334 53.3727 42.5048 58.9052 47.4212 62.6916C50.9634 57.5965 52.41 51.3341 51.4612 45.2016C50.5124 39.0691 47.241 33.5366 42.3246 29.7502Z"
							stroke="#B0BAC5"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
						/>
						<path
							d="M28.4023 48.7695L61.3437 43.673"
							stroke="#B0BAC5"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
						/>
					</g>
					<defs>
						<clipPath :id="emptyCardClipId">
							<rect
								width="40"
								height="40"
								fill="white"
								transform="translate(22.0508 29.5137) rotate(-8.79487)"
							/>
						</clipPath>
					</defs>
				</svg>
				<div class="mt-6 flex flex-col gap-1">
					<h2 class="m-0 text-2xl font-semibold leading-8 text-contrast">{{ world.name }}</h2>
					<p class="m-0 text-base leading-6 text-secondary">
						{{ formatMessage(messages.newWorldInstance) }}
					</p>
				</div>
			</div>

			<div class="px-5 pb-4">
				<ButtonStyled color="brand">
					<button class="w-full !h-10" @click="emit('create', world.id)">
						<PlusIcon aria-hidden="true" />
						{{ formatMessage(messages.createWorld) }}
					</button>
				</ButtonStyled>
			</div>
		</template>

		<template v-else>
			<header class="flex min-h-[5.75rem] flex-col justify-center gap-1 px-5 py-4">
				<div class="flex min-w-0 items-center justify-between gap-3">
					<span
						ref="worldNameRef"
						v-tooltip="truncatedTooltip(worldNameRef, world.name)"
						class="m-0 truncate text-xl font-semibold text-contrast"
						>{{ world.name }}</span
					>
					<span
						v-if="world.active"
						class="shrink-0 rounded-full bg-brand-highlight border border-brand border-solid px-2.5 py-1 text-green"
					>
						{{ formatMessage(messages.active) }}
					</span>
				</div>
				<div class="flex min-w-0 text-md items-center gap-2 text-secondary">
					{{ world.gameVersion }} <BulletDivider /> {{ world.loaderLabel }}
				</div>
			</header>

			<div
				class="flex flex-1 flex-col gap-3 border-0 border-y bg-surface-2 border-solid border-surface-5 my-auto px-5 py-4"
			>
				<div
					class="grid min-h-6 grid-cols-[minmax(0,1fr)_minmax(0,70%)] items-center gap-4 text-secondary [&>*:last-child]:max-w-full [&>*:last-child]:justify-self-end"
				>
					<span>{{ formatMessage(commonMessages.modpackLabel) }}</span>
					<div
						v-if="world.linkedModpack"
						class="flex min-w-0 items-center gap-2 text-right font-semibold leading-5 text-contrast"
					>
						<AutoLink :to="world.linkedModpack.link" class="flex shrink-0 items-center">
							<Avatar
								:src="world.linkedModpack.iconUrl"
								:alt="world.linkedModpack.name"
								:tint-by="world.linkedModpack.name"
								size="1.25rem"
								no-shadow
							/>
						</AutoLink>
						<AutoLink
							:to="world.linkedModpack.link"
							class="flex min-w-0 items-center font-semibold leading-5 text-contrast"
							:class="world.linkedModpack.link ? 'hover:underline' : ''"
						>
							<span
								ref="modpackNameRef"
								v-tooltip="truncatedTooltip(modpackNameRef, world.linkedModpack.name)"
								class="block truncate leading-5"
							>
								{{ world.linkedModpack.name }}
							</span>
						</AutoLink>
					</div>
					<span v-else class="font-semibold text-contrast">{{ formatMessage(messages.none) }}</span>
				</div>
				<div
					class="grid min-h-6 grid-cols-[minmax(0,1fr)_minmax(0,25%)] items-center gap-4 text-secondary [&>*:last-child]:max-w-full [&>*:last-child]:justify-self-end"
				>
					<span>{{ formatMessage(messages.installedContent) }}</span>
					<span class="font-semibold text-contrast">{{ installedContentLabel }}</span>
				</div>
				<div
					class="grid min-h-6 grid-cols-[minmax(0,1fr)_minmax(0,45%)] items-center gap-4 text-base font-medium text-secondary [&>*:last-child]:max-w-full [&>*:last-child]:justify-self-end"
				>
					<span>{{ formatMessage(messages.lastActive) }}</span>
					<span class="font-semibold text-contrast">{{ lastActiveLabel }}</span>
				</div>
				<div
					class="grid min-h-6 grid-cols-[minmax(0,1fr)_minmax(0,45%)] items-center gap-4 text-base font-medium text-secondary [&>*:last-child]:max-w-full [&>*:last-child]:justify-self-end"
				>
					<span>{{ formatMessage(messages.created) }}</span>
					<span class="font-semibold text-contrast">{{ createdLabel }}</span>
				</div>
			</div>

			<footer class="flex items-center justify-between gap-3 px-5 py-4">
				<ButtonStyled>
					<button class="!shadow-none" @click="emit('edit', world.id)">
						<PencilIcon aria-hidden="true" />
						{{ formatMessage(messages.editWorld) }}
					</button>
				</ButtonStyled>
				<ButtonStyled circular>
					<button
						v-tooltip="formatMessage(messages.worldSettings)"
						class="!shadow-none"
						@click="emit('settings', world.id)"
					>
						<Settings2Icon aria-hidden="true" />
					</button>
				</ButtonStyled>
			</footer>
		</template>
	</article>
</template>

<script setup lang="ts">
import { PencilIcon, PlusIcon, Settings2Icon } from '@modrinth/assets'
import { computed, useId, useTemplateRef } from 'vue'

import AutoLink from '#ui/components/base/AutoLink.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import BulletDivider from '#ui/components/base/BulletDivider.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import { useFormatDateTime, useRelativeTime } from '#ui/composables'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { truncatedTooltip } from '#ui/utils'
import { commonMessages } from '#ui/utils/common-messages'

const messages = defineMessages({
	newWorldInstance: {
		id: 'servers.manage.worlds.card.empty-description',
		defaultMessage: 'New world instance',
	},
	createWorld: {
		id: 'servers.manage.worlds.card.create',
		defaultMessage: 'Create world',
	},
	active: {
		id: 'servers.manage.worlds.card.active',
		defaultMessage: 'Active',
	},
	none: {
		id: 'servers.manage.worlds.card.none',
		defaultMessage: 'None',
	},
	installedContent: {
		id: 'servers.manage.worlds.card.installed-content',
		defaultMessage: 'Installed content',
	},
	lastActive: {
		id: 'servers.manage.worlds.card.last-active',
		defaultMessage: 'Last active',
	},
	created: {
		id: 'servers.manage.worlds.card.created',
		defaultMessage: 'Created',
	},
	editWorld: {
		id: 'servers.manage.worlds.card.edit',
		defaultMessage: 'Edit world',
	},
	worldSettings: {
		id: 'servers.manage.worlds.card.settings',
		defaultMessage: 'World settings',
	},
	notTrackedYet: {
		id: 'servers.manage.worlds.card.not-tracked-yet',
		defaultMessage: 'Not tracked yet',
	},
})

type LinkedModpack = {
	name: string
	iconUrl: string | null
	link: string | null
}

type UsedWorld = {
	type: 'world'
	id: string
	name: string
	active: boolean
	gameVersion: string | null
	loaderLabel: string | null
	linkedModpack: LinkedModpack | null
	installedContentCount: number | null
	lastActiveAt: string | null
	createdAt: string | null
}

type EmptyWorld = {
	type: 'empty'
	id: string
	name: string
}

const props = defineProps<{
	world: UsedWorld | EmptyWorld
}>()

const emit = defineEmits<{
	create: [slotId: string]
	edit: [worldId: string]
	settings: [worldId: string]
}>()

const formatRelativeTime = useRelativeTime()
const formatDate = useFormatDateTime({ dateStyle: 'medium' })
const { formatMessage } = useVIntl()
const emptyCardClipId = useId()

const modpackNameRef = useTemplateRef<HTMLElement>('modpackNameRef')
const worldNameRef = useTemplateRef<HTMLElement>('worldNameRef')

const installedContentLabel = computed(() => {
	if (props.world.type === 'empty') return ''
	return props.world.installedContentCount === null
		? formatMessage(commonMessages.unknownLabel)
		: String(props.world.installedContentCount)
})

const lastActiveLabel = computed(() => {
	if (props.world.type === 'empty') return ''
	return props.world.lastActiveAt
		? formatRelativeTime(props.world.lastActiveAt)
		: formatMessage(messages.notTrackedYet)
})

const createdLabel = computed(() => {
	if (props.world.type === 'empty') return ''
	return props.world.createdAt
		? formatDate(props.world.createdAt)
		: formatMessage(commonMessages.unknownLabel)
})
</script>
