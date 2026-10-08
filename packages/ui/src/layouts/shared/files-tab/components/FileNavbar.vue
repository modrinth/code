<template>
	<FileActionBar ref="baseRef" v-bind="{ ...$props, ...$attrs }">
		<nav
			v-if="breadcrumbs != null"
			:aria-label="formatMessage(messages.breadcrumbNavigation)"
			class="m-0 -ml-2 flex min-w-0 flex-shrink items-center p-0"
		>
			<div class="mr-1 flex shrink-0 items-center">
				<IconButton
					v-tooltip="formatMessage(messages.back)"
					type="quiet"
					:label="formatMessage(messages.back)"
					:disabled="!canGoBack"
					@click="$emit('back')"
				>
					<ChevronLeftIcon />
				</IconButton>
				<IconButton
					v-tooltip="formatMessage(messages.forward)"
					type="quiet"
					:label="formatMessage(messages.forward)"
					:disabled="!canGoForward"
					@click="$emit('forward')"
				>
					<ChevronRightIcon />
				</IconButton>
			</div>
			<ol
				ref="breadcrumbOuter"
				class="m-0 flex min-w-0 flex-shrink items-center overflow-hidden p-0"
				:class="{ 'breadcrumb-fade-mask': isBreadcrumbOverflowing }"
				:style="
					isBreadcrumbOverflowing
						? { '--scroll-distance': `-${breadcrumbOverflowAmount}px` }
						: undefined
				"
				@mouseenter="onBreadcrumbMouseEnter"
				@mouseleave="onBreadcrumbMouseLeave"
			>
				<TransitionGroup
					ref="breadcrumbInner"
					name="breadcrumb"
					tag="span"
					class="relative flex w-fit items-center"
					:class="{ 'breadcrumbs-scroll': isBreadcrumbAnimating }"
					@animationiteration="onBreadcrumbAnimationIteration"
				>
					<li :key="`home`" class="relative flex shrink-0 items-center text-sm">
						<div class="flex shrink-0 items-center">
							<Button
								v-tooltip="formatMessage(messages.backToHome)"
								:label="formatMessage(messages.backToHome)"
								type="quiet"
								class="cursor-pointer whitespace-nowrap focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand"
								:aria-current="breadcrumbs.length == 0 ? 'location' : undefined"
								:class="{ '!text-contrast': breadcrumbs.length == 0 }"
								@click="$emit('navigateHome')"
								@mouseenter="$emit('prefetchHome')"
								@contextmenu.prevent="$emit('contextmenu', $event, 0)"
							>
								<HomeIcon />
								<span>Home</span>
							</Button>
							<ChevronRightIcon
								v-if="breadcrumbs.length != 0"
								class="size-4 flex-shrink-0 text-secondary"
								aria-hidden="true"
							/>
						</div>
					</li>
					<li
						v-for="(segment, index) in breadcrumbs"
						:key="`${segment || index}-group`"
						class="relative flex shrink-0 items-center text-sm"
					>
						<div class="flex shrink-0 items-center">
							<Button
								type="quiet"
								class="cursor-pointer whitespace-nowrap focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand"
								:aria-current="index === breadcrumbs.length - 1 ? 'location' : undefined"
								:class="{ '!text-contrast': index === breadcrumbs.length - 1 }"
								@click="
									() => {
										if (index < breadcrumbs.length - 1) $emit('navigate', index + 1)
									}
								"
								@contextmenu.prevent="$emit('contextmenu', $event, index + 1)"
							>
								{{ segment || '' }}
							</Button>
							<ChevronRightIcon
								v-if="index < breadcrumbs.length - 1"
								class="size-4 flex-shrink-0 text-secondary"
								aria-hidden="true"
							/>
						</div>
					</li>
				</TransitionGroup>
			</ol>
		</nav>
	</FileActionBar>
</template>

<script setup lang="ts">
import { ChevronLeftIcon, ChevronRightIcon, HomeIcon } from '@modrinth/assets'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { Button, IconButton } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables'
import type { Properties } from '#ui/layouts/shared/files-tab/components/FileActionBar.vue'
import FileActionBar from '#ui/layouts/shared/files-tab/components/FileActionBar.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	breadcrumbNavigation: {
		id: 'files.navbar.breadcrumb-navigation',
		defaultMessage: 'Breadcrumb navigation',
	},
	backToHome: {
		id: 'files.navbar.back-to-home',
		defaultMessage: 'Back to home',
	},
	home: {
		id: 'files.navbar.home',
		defaultMessage: 'Home',
	},
	back: {
		id: 'files.navbar.back',
		defaultMessage: 'Back',
	},
	forward: {
		id: 'files.navbar.forward',
		defaultMessage: 'Forward',
	},
})

defineOptions({
	inheritAttrs: false,
})

const baseRef = ref<typeof FileActionBar | null>(null)

// Expose the base component instance so parent refs can call BaseButton methods
defineExpose({
	baseRef,
})

const props = withDefaults(
	defineProps<
		{
			canGoBack?: boolean
			canGoForward?: boolean
		} & Properties
	>(),
	{
		hasNav: true,
		sidebarToggleable: true,
	},
)

const breadcrumbs = computed(() => props.activeLocation.path.split('/').filter(Boolean))

defineEmits<{
	navigate: [index: number]
	navigateHome?: []
	prefetchHome?: []
	back: []
	forward: []
	/** Right-click on a breadcrumb; `depth` is how many path segments it covers (0 for home). */
	contextmenu: [event: MouseEvent, depth: number]
}>()

const breadcrumbOuter = ref<HTMLElement | null>(null)
const breadcrumbInner = ref<{ $el: HTMLElement } | null>(null)
const isBreadcrumbOverflowing = ref(false)
const isBreadcrumbAnimating = ref(false)
const breadcrumbOverflowAmount = ref(0)

let bcHovered = false
let bcStopping = false

function checkBreadcrumbOverflow() {
	const inner = breadcrumbInner.value?.$el
	if (!breadcrumbOuter.value || !inner) return
	const overflow = inner.scrollWidth - breadcrumbOuter.value.clientWidth
	isBreadcrumbOverflowing.value = overflow > 0
	breadcrumbOverflowAmount.value = overflow + 12
}

function onBreadcrumbMouseEnter() {
	bcHovered = true
	bcStopping = false
	if (isBreadcrumbOverflowing.value) {
		isBreadcrumbAnimating.value = true
	}
}

function onBreadcrumbMouseLeave() {
	bcHovered = false
	if (isBreadcrumbAnimating.value) {
		bcStopping = true
	}
}

function onBreadcrumbAnimationIteration() {
	if (bcStopping && !bcHovered) {
		isBreadcrumbAnimating.value = false
		bcStopping = false
	}
}

let bcResizeObserver: ResizeObserver | null = null

onMounted(() => {
	checkBreadcrumbOverflow()
	bcResizeObserver = new ResizeObserver(checkBreadcrumbOverflow)
	if (breadcrumbOuter.value) bcResizeObserver.observe(breadcrumbOuter.value)
	const innerEl = breadcrumbInner.value?.$el
	if (innerEl) bcResizeObserver.observe(innerEl)
})

onBeforeUnmount(() => {
	bcResizeObserver?.disconnect()
})

watch(
	() => props.activeLocation,
	() => {
		requestAnimationFrame(checkBreadcrumbOverflow)
	},
)
</script>

<style scoped>
.breadcrumb-move,
.breadcrumb-enter-active,
.breadcrumb-leave-active {
	transition: all 0.2s ease;
}

.breadcrumb-enter-from {
	opacity: 0;
	transform: translateX(-10px) scale(0.9);
}

.breadcrumb-leave-to {
	opacity: 0;
	transform: translateX(-10px) scale(0.8);
	filter: blur(4px);
}

.breadcrumb-leave-active {
	position: relative;
	pointer-events: none;
}

.breadcrumb-move {
	z-index: 1;
}

.breadcrumb-fade-mask {
	mask-image: linear-gradient(
		to right,
		transparent,
		black 12px,
		black calc(100% - 12px),
		transparent
	);
}

.breadcrumbs-scroll {
	animation: breadcrumb-scroll 10s ease-in-out infinite;
}

@keyframes breadcrumb-scroll {
	0% {
		transform: translateX(0);
	}
	35%,
	65% {
		transform: translateX(var(--scroll-distance));
	}
	100% {
		transform: translateX(0);
	}
}
</style>
