<script setup>
import { SpinnerIcon } from '@modrinth/assets'
import { Avatar, defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import { computed, onMounted, onUnmounted, ref } from 'vue'

import NavButton from '@/components/ui/NavButton.vue'
import { useAppEvent } from '@/composables/use-app-event'
import { getInstanceIconUrl, list } from '@/helpers/instance'
import { instanceKeys } from '@/pages/instance/query-options'

const ITEM_SIZE = 52
const APPROX_USED_VERTICAL_SPACE = 475 // doesn't need to be exact lol just close enough so there's a little gap and no overflow
const STORAGE_KEY = 'modrinth-quick-instance-count'

const { handleError } = injectNotificationManager()
const queryClient = useQueryClient()

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

onMounted(() => {
	window.addEventListener('resize', updateMaxAuto)
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
})

const dividerTooltip = computed(() => {
	if (!canDrag.value || dragging.value) {
		return null
	}
	return formatMessage(visibleCount.value === 0 ? messages.dragShowTooltip : messages.dragTooltip)
})
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
		>
			<NavButton :to="`/instance/${encodeURIComponent(instance.id)}`" class="relative">
				<Avatar
					:src="getInstanceIconUrl(instance.icon_path)"
					size="28px"
					:tint-by="instance.id"
					:class="`transition-all ${instance.install_stage !== 'installed' ? `brightness-[0.25] scale-[0.85]` : `group-hover:brightness-75`}`"
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
