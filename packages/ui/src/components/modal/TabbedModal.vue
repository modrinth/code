<script lang="ts">
export interface Tab<Props> {
	name: MessageDescriptor
	icon: Component
	content: Component
	props?: Props
	badge?: MessageDescriptor
	shown?: boolean
}
</script>

<script setup lang="ts" generic="T">
import { type Component, computed, nextTick, ref } from 'vue'

import { type MessageDescriptor, useVIntl } from '../../composables/i18n'
import { useScrollIndicator } from '../../composables/scroll-indicator'
import NewModal from './NewModal.vue'

const { formatMessage } = useVIntl()

const props = withDefaults(
	defineProps<{
		tabs: Tab<T>[]
		header?: string
		maxWidth?: string
		width?: string
		closable?: boolean
		onHide?: () => void
		onShow?: () => void
	}>(),
	{
		header: undefined,
		maxWidth: undefined,
		width: undefined,
		closable: true,
		onHide: undefined,
		onShow: undefined,
	},
)

const visibleTabs = computed(() => props.tabs.filter((tab) => tab.shown !== false))

const selectedTab = ref(0)

const scrollContainer = ref<HTMLElement | null>(null)
const { showTopFade, showBottomFade, checkScrollState, forceCheck } =
	useScrollIndicator(scrollContainer)

const modal = ref<InstanceType<typeof NewModal> | null>(null)

function setTab(index: number) {
	selectedTab.value = index
	nextTick(() => forceCheck())
}

function show(event?: MouseEvent) {
	modal.value?.show(event)
}

function hide() {
	modal.value?.hide()
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
		no-padding
	>
		<template v-if="$slots.title" #title>
			<slot name="title" />
		</template>
		<div class="grid grid-cols-[auto_1fr] p-4">
			<div
				class="flex flex-col gap-1 border-solid pr-4 border-0 border-r-[1px] border-divider min-w-[200px]"
			>
				<button
					v-for="(tab, index) in visibleTabs"
					:key="index"
					:class="`flex gap-2 items-center text-left rounded-xl px-4 py-2 border-none text-nowrap font-semibold cursor-pointer active:scale-[0.97] transition-all ${selectedTab === index ? 'bg-button-bgSelected text-button-textSelected' : 'bg-transparent text-button-text hover:bg-button-bg hover:text-contrast'}`"
					@click="() => setTab(index)"
				>
					<component :is="tab.icon" class="w-4 h-4 flex-shrink-0" />
					<span>{{ formatMessage(tab.name) }}</span>
					<span
						v-if="tab.badge"
						class="rounded-full px-1.5 py-0.5 text-xs font-bold bg-brand-highlight text-brand-green"
					>
						{{ formatMessage(tab.badge) }}
					</span>
				</button>

				<slot name="footer" />
			</div>
			<div class="relative">
				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-10"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-10"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showTopFade"
						class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-10 bg-gradient-to-b from-bg-raised to-transparent"
					/>
				</Transition>

				<div
					ref="scrollContainer"
					class="min-w-[400px] h-[500px] overflow-y-auto px-4"
					@scroll="checkScrollState"
				>
					<Suspense>
						<component
							:is="visibleTabs[selectedTab].content"
							v-bind="visibleTabs[selectedTab].props ?? {}"
						/>
					</Suspense>
				</div>

				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-10"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-10"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showBottomFade"
						class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-10 bg-gradient-to-t from-bg-raised to-transparent"
					/>
				</Transition>
			</div>
		</div>
	</NewModal>
</template>
