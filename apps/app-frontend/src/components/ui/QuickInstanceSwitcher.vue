<script setup>
import { EyeIcon, FolderOpenIcon, PlayIcon, SpinnerIcon, StopCircleIcon } from '@modrinth/assets'
import {
	Avatar,
	commonMessages,
	ContextMenu,
	defineMessages,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import NavButton from '@/components/ui/NavButton.vue'
import { useAppEvent } from '@/composables/use-app-event'
import { handleSevereError } from '@/composables/use-error.js'
import { trackEvent } from '@/helpers/analytics'
import { getInstanceIconUrl, kill, list, run } from '@/helpers/instance'
import { get_all } from '@/helpers/process'
import { showInstanceInFolder } from '@/helpers/utils'
import { instanceKeys } from '@/pages/instance/query-options'

const ITEM_SIZE = 52
const APPROX_USED_VERTICAL_SPACE = 475 // doesn't need to be exact lol just close enough so there's a little gap and no overflow
const STORAGE_KEY = 'modrinth-quick-instance-count'

const { handleError } = injectNotificationManager()
const queryClient = useQueryClient()
const router = useRouter()
const instanceOptions = ref()
const runningInstances = ref([])

const { formatMessage } = useVIntl()

const maxAuto = ref(0)
const allInstances = ref([])
const dragging = ref(false)

const stored = localStorage.getItem(STORAGE_KEY)
const userLimit = ref(stored === null ? null : Number(stored))

const maxVisible = computed(() => Math.min(maxAuto.value, allInstances.value.length))
const visibleCount = computed(() => Math.min(userLimit.value ?? maxVisible.value, maxVisible.value))
const recentInstances = computed(() => allInstances.value.slice(0, visibleCount.value))
const canDrag = computed(() => maxVisible.value > 0)
const showOverdrag = ref(false)

const updateMaxAuto = () => {
	maxAuto.value = Math.max(
		0,
		Math.floor((window.innerHeight - APPROX_USED_VERTICAL_SPACE) / ITEM_SIZE),
	)
}

const setLimit = (count) => {
	const clamped = Math.max(0, Math.min(count, maxVisible.value))
	if (clamped >= maxVisible.value) {
		userLimit.value = null
		localStorage.removeItem(STORAGE_KEY)
	} else {
		userLimit.value = clamped
		localStorage.setItem(STORAGE_KEY, String(clamped))
	}
}

let dragStartY = 0
let dragStartCount = 0
let wasOverdragging = false
let overdragTimeout = null

const clearOverdragFlash = () => {
	showOverdrag.value = false
	if (overdragTimeout !== null) {
		clearTimeout(overdragTimeout)
		overdragTimeout = null
	}
}

const flashOverdrag = () => {
	showOverdrag.value = true
	if (overdragTimeout !== null) {
		clearTimeout(overdragTimeout)
	}
	overdragTimeout = setTimeout(() => {
		showOverdrag.value = false
		overdragTimeout = null
	}, 500)
}

const onDividerPointerDown = (event) => {
	if (!canDrag.value) {
		return
	}
	event.preventDefault()
	dragging.value = true
	wasOverdragging = false
	clearOverdragFlash()
	dragStartY = event.clientY
	dragStartCount = visibleCount.value
	document.body.classList.add('quick-instance-dragging')
	event.currentTarget.setPointerCapture(event.pointerId)
}

const onDividerPointerMove = (event) => {
	if (!dragging.value) {
		return
	}
	const delta = event.clientY - dragStartY
	const target = dragStartCount + Math.round(delta / ITEM_SIZE)
	const isOverdragging = target < 0 || target > maxAuto.value
	if (isOverdragging && !wasOverdragging) {
		flashOverdrag()
	}
	wasOverdragging = isOverdragging
	setLimit(target)
}

const endDrag = (event) => {
	if (!dragging.value) {
		return
	}
	dragging.value = false
	wasOverdragging = false
	clearOverdragFlash()
	document.body.classList.remove('quick-instance-dragging')
	if (event?.currentTarget?.hasPointerCapture?.(event.pointerId)) {
		event.currentTarget.releasePointerCapture(event.pointerId)
	}
}

const onDividerPointerUp = (event) => {
	endDrag(event)
}

const getInstances = async () => {
	const instances = await list().catch(handleError)

	for (const instance of instances) {
		queryClient.setQueryData(instanceKeys.detail(instance.id), instance)
	}

	allInstances.value = instances.sort((a, b) => {
		const dateACreated = dayjs(a.created)
		const dateAPlayed = a.last_played ? dayjs(a.last_played) : dayjs(0)

		const dateBCreated = dayjs(b.created)
		const dateBPlayed = b.last_played ? dayjs(b.last_played) : dayjs(0)

		const dateA = dateACreated.isAfter(dateAPlayed) ? dateACreated : dateAPlayed
		const dateB = dateBCreated.isAfter(dateBPlayed) ? dateBCreated : dateBPlayed

		if (dateA.isSame(dateB)) {
			return a.name.localeCompare(b.name)
		}

		return dateB - dateA
	})
}

await getInstances()
updateMaxAuto()

useAppEvent('instance', async (event) => {
	if (event.event !== 'synced') {
		await getInstances()
	}
})

useAppEvent('process', checkProcesses)

onMounted(() => {
	window.addEventListener('resize', updateMaxAuto)
	checkProcesses()
})

onUnmounted(() => {
	window.removeEventListener('resize', updateMaxAuto)
	document.body.classList.remove('quick-instance-dragging')
	clearOverdragFlash()
})

const messages = defineMessages({
	dragTooltip: {
		id: 'app.quick-instance-switcher.drag-tooltip',
		defaultMessage: 'Drag to resize',
	},
	dragShowTooltip: {
		id: 'app.quick-instance-switcher.drag-show-tooltip',
		defaultMessage: 'Drag to show recent instances',
	},
	viewInstance: {
		id: 'app.quick-instance-switcher.view-instance',
		defaultMessage: 'View instance',
	},
	instanceActions: {
		id: 'app.quick-instance-switcher.actions.label',
		defaultMessage: 'Instance actions',
	},
	instanceLocked: {
		id: 'app.quick-instance-switcher.instance-locked',
		defaultMessage: 'This instance has been locked',
	},
})

const dividerTooltip = computed(() => {
	if (!canDrag.value || dragging.value) {
		return null
	}
	return formatMessage(visibleCount.value === 0 ? messages.dragShowTooltip : messages.dragTooltip)
})

async function checkProcesses() {
	const processes = (await get_all().catch(handleError)) ?? []
	runningInstances.value = processes.map((process) => process.instance_id)
}

async function playInstance(instance) {
	if (instance.quarantined || instance.install_stage !== 'installed') return
	await run(instance.id)
		.catch((err) => handleSevereError(err, { instanceId: instance.id }))
		.finally(() => {
			trackEvent('InstanceStart', {
				loader: instance.loader,
				game_version: instance.game_version,
				source: 'QuickInstanceSwitcher',
			})
		})
}

async function stopInstance(instance) {
	await kill(instance.id).catch(handleError)
	trackEvent('InstanceStop', {
		loader: instance.loader,
		game_version: instance.game_version,
		source: 'QuickInstanceSwitcher',
	})
}

function openContextMenu(event, instance) {
	const playing = runningInstances.value.includes(instance.id)
	instanceOptions.value?.open(event, [
		playing
			? {
					id: 'stop',
					label: formatMessage(commonMessages.stopButton),
					icon: StopCircleIcon,
					tone: 'red',
					action: () => stopInstance(instance),
				}
			: {
					id: 'play',
					label: formatMessage(commonMessages.playButton),
					icon: PlayIcon,
					tone: 'brand',
					disabled: instance.quarantined || instance.install_stage !== 'installed',
					tooltip: instance.quarantined ? formatMessage(messages.instanceLocked) : undefined,
					action: () => playInstance(instance),
				},
		{ type: 'divider' },
		{
			id: 'open-instance',
			label: formatMessage(messages.viewInstance),
			icon: EyeIcon,
			action: () => router.push(`/instance/${encodeURIComponent(instance.id)}`),
		},
		{
			id: 'open-folder',
			label: formatMessage(commonMessages.openFolderButton),
			icon: FolderOpenIcon,
			action: () => showInstanceInFolder(instance.id),
		},
	])
}
</script>

<template>
	<Transition name="top-divider">
		<div
			v-if="recentInstances.length > 0"
			class="top-divider flex items-center justify-center overflow-hidden"
		>
			<div class="h-px w-8 bg-surface-5 shrink-0"></div>
		</div>
	</Transition>
	<TransitionGroup name="quick-instance" tag="div" class="flex flex-col items-center">
		<div
			v-for="instance in recentInstances"
			:key="instance.id"
			v-tooltip.right="instance.name"
			class="quick-instance-item"
			@contextmenu.prevent.stop="(event) => openContextMenu(event, instance)"
		>
			<NavButton :to="`/instance/${encodeURIComponent(instance.id)}`" class="relative">
				<Avatar
					:src="getInstanceIconUrl(instance.icon_path)"
					size="28px"
					:tint-by="instance.id"
					:class="`transition-all ${instance.install_stage !== 'installed' ? `brightness-[0.25] scale-[0.85]` : `group-hover:brightness-75`}`"
					pad-transparent-corners
				/>
				<div
					v-if="instance.install_stage !== 'installed'"
					class="absolute inset-0 flex items-center justify-center z-10 pointer-events-none"
				>
					<SpinnerIcon class="animate-spin w-4 h-4" />
				</div>
			</NavButton>
		</div>
	</TransitionGroup>
	<ContextMenu ref="instanceOptions" :label="formatMessage(messages.instanceActions)" />
	<div
		v-tooltip.right="dividerTooltip"
		class="flex items-center justify-center py-2 select-none"
		:class="canDrag ? 'cursor-ns-resize touch-none group' : ''"
		@pointerdown="onDividerPointerDown"
		@pointermove="onDividerPointerMove"
		@pointerup="onDividerPointerUp"
		@pointercancel="onDividerPointerUp"
	>
		<div
			class="h-px w-8 transition-colors duration-200"
			:class="
				showOverdrag ? 'bg-red' : canDrag ? 'bg-surface-5 group-hover:bg-secondary' : 'bg-surface-5'
			"
		></div>
	</div>
</template>

<style scoped lang="scss">
.top-divider {
	height: calc(1rem + 1px);
}

.top-divider-enter-active,
.top-divider-leave-active {
	transition:
		opacity 0.25s ease,
		height 0.25s ease;
}

.top-divider-enter-from,
.top-divider-leave-to {
	opacity: 0;
	height: 0;
}

.quick-instance-item {
	height: 3rem;
	overflow: hidden;

	& + & {
		margin-top: 0.25rem;
	}
}

.quick-instance-enter-active,
.quick-instance-leave-active {
	transition:
		opacity 0.25s ease,
		transform 0.25s ease,
		height 0.25s ease,
		margin-top 0.25s ease;
}

.quick-instance-enter-from,
.quick-instance-leave-to {
	opacity: 0;
	transform: scale(0.5);
	height: 0;
	margin-top: 0 !important;
}

@media (prefers-reduced-motion: reduce) {
	.top-divider-enter-active,
	.top-divider-leave-active,
	.quick-instance-enter-active,
	.quick-instance-leave-active {
		transition: none;
	}

	.top-divider-enter-from,
	.top-divider-leave-to {
		opacity: 1;
		height: calc(1rem + 1px);
	}

	.quick-instance-enter-from,
	.quick-instance-leave-to {
		opacity: 1;
		transform: none;
		height: 3rem;
		margin-top: unset !important;
	}
}
</style>

<style lang="scss">
body.quick-instance-dragging,
body.quick-instance-dragging * {
	cursor: ns-resize !important;
}
</style>
