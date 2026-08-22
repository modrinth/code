<script lang="ts"></script>

<script setup lang="ts">
import { RightArrowIcon } from '@modrinth/assets'
import { type Component, type ComponentPublicInstance, computed, nextTick, ref } from 'vue'

import { type MessageDescriptor, useVIntl } from '../../composables/i18n'
import { useScrollIndicator } from '../../composables/scroll-indicator'
import { truncatedTooltip } from '../../utils/truncate'
import NewModal from './NewModal.vue'
export interface Tab {
	name: MessageDescriptor
	category?: MessageDescriptor
	icon: Component
	content?: Component
	href?: string
	badge?: MessageDescriptor
	shown?: boolean
}

const { formatMessage } = useVIntl()

const props = withDefaults(
	defineProps<{
		tabs: Tab[]
		header?: string
		maxWidth?: string
		width?: string
		closable?: boolean
		onHide?: () => void
		onShow?: () => void
		beforeHide?: () => boolean
		beforeTabChange?: (fromIndex: number, toIndex: number) => boolean
		floatingActionBarShown?: boolean
	}>(),
	{
		header: undefined,
		maxWidth: undefined,
		width: undefined,
		closable: true,
		onHide: undefined,
		onShow: undefined,
		beforeHide: undefined,
		beforeTabChange: undefined,
		floatingActionBarShown: false,
	},
)

const visibleTabs = computed(() => props.tabs.filter((tab) => tab.shown !== false))

const selectedTab = ref(0)
const tabLabelRefs = ref<Record<number, HTMLElement | null>>({})

function setTabLabelRef(index: number, element: Element | ComponentPublicInstance | null) {
	tabLabelRefs.value[index] = element instanceof HTMLElement ? element : null
}

function tabLabelTooltip(index: number, label: string) {
	return truncatedTooltip(tabLabelRefs.value[index], label)
}

const scrollContainer = ref<HTMLElement | null>(null)
const { showTopFade, showBottomFade, checkScrollState, forceCheck } =
	useScrollIndicator(scrollContainer)

const sidebarScrollContainer = ref<HTMLElement | null>(null)
const {
	showTopFade: showSidebarTopFade,
	showBottomFade: showSidebarBottomFade,
	checkScrollState: checkSidebarScrollState,
} = useScrollIndicator(sidebarScrollContainer)

const modal = ref<InstanceType<typeof NewModal> | null>(null)

function setTab(index: number) {
	if (index === selectedTab.value) return
	if (props.beforeTabChange?.(selectedTab.value, index) === false) return
	selectedTab.value = index
	nextTick(() => forceCheck())
}

function show(event?: MouseEvent) {
	modal.value?.show(event)
}

function hide(): boolean {
	return modal.value?.hide() ?? false
}

function startsCategory(index: number) {
	const category = visibleTabs.value[index]?.category
	return !!category && category.id !== visibleTabs.value[index - 1]?.category?.id
}

defineExpose({ show, hide, selectedTab, setTab })
</script>
<template>
	<NewModal
		ref="modal"
		:header="header"
		:max-width="maxWidth"
		:width="width"
		:closable="closable"
		:on-hide="onHide"
		:on-show="onShow"
		:before-hide="beforeHide"
		no-padding
	>
		<template v-if="$slots.title" #title>
			<slot name="title" />
		</template>
		<div class="grid grid-cols-[minmax(12.5rem,18rem)_minmax(0,1fr)] p-6 pb-3 pr-0">
			<div
				class="flex min-w-0 max-h-[min(65vh,600px)] flex-col border-0 border-r-[1px] border-solid border-divider pr-4"
			>
				<div class="relative min-h-0 flex-1">
					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-4"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-4"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showSidebarTopFade"
							class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-4 bg-gradient-to-b from-bg-raised to-transparent"
						/>
					</Transition>

					<div
						ref="sidebarScrollContainer"
						class="flex h-full flex-col gap-1 overflow-y-auto"
						@scroll="checkSidebarScrollState"
					>
						<template v-for="(tab, index) in visibleTabs" :key="index">
							<div
								v-if="startsCategory(index) && tab.category"
								class="truncate px-4 pb-1 pt-2 text-xs font-bold uppercase tracking-wide text-secondary"
							>
								{{ formatMessage(tab.category) }}
							</div>
							<component
								:is="tab.href ? 'a' : 'button'"
								:href="tab.href ?? undefined"
								:target="tab.href ? '_blank' : undefined"
								:rel="tab.href ? 'noopener noreferrer' : undefined"
								:class="`flex min-w-0 gap-2 items-center text-left rounded-xl px-4 py-2 border-none font-semibold cursor-pointer active:scale-[0.97] transition-all no-underline ${!tab.href && selectedTab === index ? 'bg-button-bgSelected text-button-textSelected' : 'bg-transparent text-button-text hover:bg-button-bg hover:text-contrast'}`"
								@click="!tab.href && setTab(index)"
							>
								<component :is="tab.icon" class="w-4 h-4 flex-shrink-0" />
								<span
									:ref="(element) => setTabLabelRef(index, element)"
									v-tooltip="tabLabelTooltip(index, formatMessage(tab.name))"
									class="min-w-0 flex-1 truncate"
								>
									{{ formatMessage(tab.name) }}
								</span>
								<span
									v-if="tab.badge"
									class="shrink-0 rounded-full px-1.5 py-0.5 text-xs font-bold bg-brand-highlight text-brand-green"
								>
									{{ formatMessage(tab.badge) }}
								</span>
								<RightArrowIcon v-if="tab.href" class="ml-auto size-4 shrink-0" />
							</component>
						</template>
					</div>

					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-16"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-16"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showSidebarBottomFade"
							class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-16 bg-gradient-to-t from-bg-raised to-transparent"
						/>
					</Transition>
				</div>

				<slot name="footer" />
			</div>
			<div class="relative min-h-[min(65vh,600px)]">
				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-4"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-4"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showTopFade"
						class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-4 bg-gradient-to-b from-bg-raised to-transparent"
					/>
				</Transition>

				<div
					ref="scrollContainer"
					class="absolute inset-0 overflow-y-auto px-6"
					:class="floatingActionBarShown ? 'pb-24' : 'pb-6'"
					@scroll="checkScrollState"
				>
					<Suspense>
						<component
							:is="visibleTabs[selectedTab]?.content"
							v-if="visibleTabs[selectedTab]?.content"
						/>
					</Suspense>
				</div>

				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-16"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-16"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showBottomFade"
						class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-16 bg-gradient-to-t from-bg-raised to-transparent"
					/>
				</Transition>

				<div class="pointer-events-none absolute bottom-3 left-6 right-6 z-20">
					<div class="pointer-events-auto">
						<slot name="floating-action-bar" />
					</div>
				</div>
			</div>
		</div>
	</NewModal>
</template>
