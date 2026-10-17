<template>
	<footer class="shrink-0 border-0 border-t border-solid border-divider text-sm">
		<div v-if="navigationError" role="alert" class="px-3 py-2 text-red">
			{{ formatMessage(messages.queueError) }}
		</div>
		<nav
			class="flex flex-wrap items-center justify-between gap-x-3 gap-y-1 px-2 pr-2.5 text-secondary"
			:aria-label="formatMessage(messages.queueNavigation)"
			:aria-busy="busy"
		>
			<div class="flex items-center gap-3 font-medium" aria-live="polite">
				<div class="flex">
					<button
						v-tooltip="{ text: formatMessage(commonMessages.settingsLabel), delay: 500 }"
						type="button"
						class="queue-action"
						:aria-label="formatMessage(commonMessages.settingsLabel)"
						@click="settingsModal?.show($event)"
					>
						<SettingsIcon aria-hidden="true" />
					</button>
					<FloatingMenu
						placement="top-start"
						@open="panelMenuOpen = true"
						@close="panelMenuOpen = false"
					>
						<button
							v-tooltip="{ text: formatMessage(messages.panelLayout), delay: 500 }"
							type="button"
							class="queue-action"
							:aria-label="formatMessage(messages.panelLayout)"
							:aria-expanded="panelMenuOpen"
						>
							<LayoutPanelLeftIcon aria-hidden="true" />
						</button>
						<template #popper="{ hide }">
							<div class="flex flex-col gap-1">
								<Button
									type="quiet"
									class="justify-start"
									:disabled="isLoading || poppingOut.length > 0"
									@click="runPanelAction(resetPanelLayout, hide)"
								>
									<RotateCounterClockwiseIcon aria-hidden="true" />
									{{ formatMessage(messages.resetPanelLayout) }}
								</Button>
								<Button
									v-if="hasPopouts"
									type="quiet"
									class="justify-start"
									@click="runPanelAction(closeExternalPanels, hide)"
								>
									<MonitorXIcon aria-hidden="true" />
									{{ formatMessage(messages.closePopoutWindows) }}
								</Button>
							</div>
						</template>
					</FloatingMenu>
				</div>

				<div class="flex items-center gap-2">
					<template v-for="(column, index) in queueColumns" :key="column.label.id">
						<span v-if="index" class="h-5 w-px bg-divider" aria-hidden="true" />
						<FloatingMenu trigger="hover" placement="top-start" theme="tooltip" :distance="4">
							<span tabindex="0" class="cursor-default pb-1 pt-1.5 text-xs">
								{{ formatMessage(column.label) }}
								<span class="ml-1 tabular-nums text-primary">{{ column.count }}</span>
							</span>
							<template #popper>
								<div class="flex max-w-80 flex-col gap-2 py-1">
									<span class="font-semibold">{{ formatMessage(column.heading) }}</span>
									<QueueProjectList :project-ids="column.projects" :empty-message="column.empty" />
								</div>
							</template>
						</FloatingMenu>
					</template>
				</div>
			</div>
			<div class="flex items-center gap-1">
				<button type="button" class="queue-action" :disabled="busy" @click="exit">
					{{ formatMessage(messages.exit) }}
				</button>
				<span class="mx-1.5 h-5 w-px bg-divider" aria-hidden="true" />
				<button type="button" class="queue-action" :disabled="disabled || !canGoBack" @click="back">
					<LeftArrowIcon class="mb-px" />
					{{ formatMessage(messages.back) }}
				</button>
				<button
					type="button"
					class="queue-action"
					:disabled="disabled || (!inQueue && !completed)"
					@click="next"
				>
					{{
						formatMessage(
							completed ? (remaining.length ? messages.next : messages.finish) : messages.skip,
						)
					}}
					<RightArrowIcon class="mb-px" />
				</button>
			</div>
		</nav>
	</footer>
	<TabbedModal
		ref="settingsModal"
		:header="formatMessage(commonMessages.settingsLabel)"
		:tabs="settingsTabs"
		width="60rem"
	>
		<template v-if="!isLocal" #footer>
			<p class="m-0 shrink-0 break-words text-xs text-secondary">
				{{ formatMessage(deploymentMessages.basedOn) }}
				<a
					v-if="config.public.owner && config.public.branch"
					class="hover:underline"
					target="_blank"
					rel="noopener"
					:href="`https://github.com/${config.public.owner}/code/tree/${config.public.branch}`"
				>
					{{ config.public.owner }}/{{ config.public.branch }}
				</a>
				@
				<span v-if="config.public.hash === 'unknown'">{{
					formatMessage(deploymentMessages.unknownCommit)
				}}</span>
				<AutoLink
					v-else
					class="text-link"
					target="_blank"
					:to="`https://github.com/${config.public.owner}/code/commit/${config.public.hash}`"
				>
					{{ config.public.hash }}
				</AutoLink>
			</p>
		</template>
		<template #content="{ tab }">
			<component :is="tab.content" v-if="tab?.content" embedded />
		</template>
	</TabbedModal>
</template>

<script setup lang="ts">
import {
	KeyboardIcon,
	LayoutPanelLeftIcon,
	LeftArrowIcon,
	MonitorXIcon,
	RightArrowIcon,
	RotateCounterClockwiseIcon,
	SettingsIcon,
} from '@modrinth/assets'
import {
	AutoLink,
	Button,
	commonMessages,
	defineMessages,
	FloatingMenu,
	TabbedModal,
	type TabbedModalTab,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import ModerationKeybinds from '~/components/ui/moderation/settings/ModerationKeybinds.vue'
import ModerationSettings from '~/components/ui/moderation/settings/ModerationSettings.vue'
import { injectProjectReviewPageContext } from '~/providers/project-review'

import { injectProjectReviewContext } from './layout/context'
import { projectReviewMessages as messages } from './messages'
import QueueProjectList from './queue-project-list.vue'
import { useReviewShortcut } from './shortcuts'

const config = useRuntimeConfig()
const { hostname } = useRequestURL()
const isLocal =
	import.meta.dev ||
	hostname === 'localhost' ||
	hostname.endsWith('.localhost') ||
	hostname.startsWith('127.') ||
	hostname === '[::1]'
const deploymentMessages = defineMessages({
	basedOn: { id: 'moderation.settings.based-on', defaultMessage: 'Based on' },
	unknownCommit: { id: 'moderation.settings.unknown-commit', defaultMessage: 'unknown' },
})

const panelMenuOpen = ref(false)
const settingsModal = ref<InstanceType<typeof TabbedModal> | null>(null)
const settingsTabs: TabbedModalTab[] = [
	{
		name: messages.generalSettings,
		icon: SettingsIcon,
		content: ModerationSettings,
	},
	{
		name: messages.keybindsSettings,
		icon: KeyboardIcon,
		content: ModerationKeybinds,
	},
]

const { queue, isLoading, navigation } = injectProjectReviewPageContext()
const { hasPopouts, closeExternalPanels, resetPanelLayout, poppingOut } =
	injectProjectReviewContext()
const {
	busy,
	error: navigationError,
	canGoBack,
	inQueue,
	completed,
	remaining,
	back,
	next,
	exit,
} = navigation
const { formatMessage } = useVIntl()
const disabled = computed(() => busy.value || isLoading.value || !queue.hydrated)
const queueColumns = computed(() => {
	const current = queue.currentQueue
	const completedIds = new Set(current.completed)
	const skippedIds = new Set(current.skipped)
	return [
		{
			label: messages.complete,
			heading: messages.completedProjects,
			empty: messages.noCompletedProjects,
			count: current.completed.length,
			projects: current.projectOrder.filter((id) => completedIds.has(id)),
		},
		{
			label: messages.skipped,
			heading: messages.skippedProjects,
			empty: messages.noSkippedProjects,
			count: current.skipped.length,
			projects: current.projectOrder.filter((id) => skippedIds.has(id)),
		},
		{
			label: messages.remaining,
			heading: messages.queuedProjects,
			empty: messages.noQueuedProjects,
			count: current.items.length,
			projects: remaining.value.slice(0, 5),
		},
	]
})

function runPanelAction(action: () => void, hide: () => void) {
	action()
	hide()
}

useReviewShortcut(
	'back',
	() => {
		void back()
	},
	() => !disabled.value && canGoBack.value,
)
useReviewShortcut(
	'next',
	() => {
		void next()
	},
	() => !disabled.value && (inQueue.value || completed.value),
)
useReviewShortcut(
	'exit',
	() => {
		void exit()
	},
	() => !busy.value,
)
</script>

<style scoped>
.queue-action {
	display: inline-flex;
	align-items: center;
	gap: 0.25rem;
	padding: 0.625rem 0.375rem;
	background: transparent;
	color: inherit;
	font: inherit;
	font-size: inherit;
	font-weight: 600;
}
.queue-action:hover:not(:disabled) {
	color: var(--color-contrast);
}
.queue-action:disabled {
	cursor: default;
	opacity: 0.4;
}
.queue-action :deep(svg) {
	width: 1rem;
	height: 1rem;
}
</style>
