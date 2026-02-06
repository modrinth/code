<script setup lang="ts">
/**
 * "I don't know why, I don't want to know why, I shouldn't have to wonder why"
 */

import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'

import {
	EyeIcon,
	EyeOffIcon,
	MaximizeIcon,
	MinusIcon,
	ScanEyeIcon,
	SearchIcon,
	XIcon,
} from '@modrinth/assets'

import { injectI18nDebug } from '../../composables/i18n-debug'
import ButtonStyled from './ButtonStyled.vue'

const debugContext = injectI18nDebug()

const searchQuery = ref('')
const minimized = ref(false)
const copiedKey = ref<string | null>(null)
const highlightedEl = ref<Element | null>(null)
const searchInputRef = ref<HTMLInputElement | null>(null)
const activeEntryIndex = ref(-1)
const listContainerRef = ref<HTMLElement | null>(null)

// Dragging state
const isDragging = ref(false)
const panelPos = ref({ x: -1, y: -1 })
const dragOffset = ref({ x: 0, y: 0 })

// Resize state
const isResizing = ref(false)
const panelWidth = ref(380)
const panelHeight = ref(420)
const resizeStart = ref({ x: 0, y: 0, w: 0, h: 0 })

const filteredEntries = computed(() => {
	if (!debugContext) return []
	const entries = Array.from(debugContext.registry.values())
	const q = searchQuery.value.toLowerCase()
	if (!q) return entries
	return entries.filter((e) => e.key.toLowerCase().includes(q) || e.value.toLowerCase().includes(q))
})

const keyCount = computed(() => debugContext?.registry.size ?? 0)
const matchCount = computed(() => filteredEntries.value.length)

// Reset active index when search changes
watch(searchQuery, () => {
	activeEntryIndex.value = -1
})

function truncate(str: string, max: number): string {
	return str.length > max ? str.slice(0, max) + '\u2026' : str
}

function highlightMatch(text: string, query: string): string {
	if (!query) return escapeHtml(text)
	const escaped = escapeHtml(text)
	const q = escapeHtml(query)
	const regex = new RegExp(`(${q.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi')
	return escaped.replace(regex, '<mark class="bg-brand/20 text-brand rounded-sm px-0.5">$1</mark>')
}

function escapeHtml(str: string): string {
	return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

function toggleKeyReveal() {
	if (debugContext) {
		debugContext.keyReveal.value = !debugContext.keyReveal.value
	}
}

function toggleOverlay() {
	if (debugContext?.enabled.value) {
		document.body.classList.toggle('i18n-debug')
	}
}

function closePanel() {
	if (debugContext) {
		debugContext.panelOpen.value = false
	}
}

function highlightElement(key: string) {
	clearHighlight()
	const el = document.querySelector(`[data-i18n-key="${CSS.escape(key)}"]`)
	if (el) {
		highlightedEl.value = el
		el.scrollIntoView({ behavior: 'smooth', block: 'center' })
		;(el as HTMLElement).style.outline = '2px solid var(--color-brand)'
		;(el as HTMLElement).style.outlineOffset = '3px'
		;(el as HTMLElement).style.borderRadius = '4px'
	}
}

function clearHighlight() {
	if (highlightedEl.value) {
		;(highlightedEl.value as HTMLElement).style.outline = ''
		;(highlightedEl.value as HTMLElement).style.outlineOffset = ''
		;(highlightedEl.value as HTMLElement).style.borderRadius = ''
		highlightedEl.value = null
	}
}

async function copyKey(key: string) {
	try {
		await navigator.clipboard.writeText(key)
		copiedKey.value = key
		setTimeout(() => {
			copiedKey.value = null
		}, 2000)
	} catch {
		// clipboard not available
	}
}

function onPanelKeydown(e: KeyboardEvent) {
	if (e.key === 'ArrowDown') {
		e.preventDefault()
		activeEntryIndex.value = Math.min(activeEntryIndex.value + 1, filteredEntries.value.length - 1)
		scrollActiveIntoView()
	} else if (e.key === 'ArrowUp') {
		e.preventDefault()
		activeEntryIndex.value = Math.max(activeEntryIndex.value - 1, 0)
		scrollActiveIntoView()
	} else if (e.key === 'Enter' && activeEntryIndex.value >= 0) {
		e.preventDefault()
		const entry = filteredEntries.value[activeEntryIndex.value]
		if (entry) copyKey(entry.key)
	} else if (e.key === 'Escape') {
		if (searchQuery.value) {
			searchQuery.value = ''
		} else {
			closePanel()
		}
	} else if (e.key === '/' && document.activeElement !== searchInputRef.value) {
		e.preventDefault()
		searchInputRef.value?.focus()
	}
}

function scrollActiveIntoView() {
	nextTick(() => {
		const activeEl = listContainerRef.value?.querySelector('[data-active="true"]')
		activeEl?.scrollIntoView({ block: 'nearest' })
	})
}

// Drag handling
function onHeaderMouseDown(e: MouseEvent) {
	if ((e.target as HTMLElement).closest('button')) return
	isDragging.value = true
	const panel = (e.currentTarget as HTMLElement).closest('.i18n-debug-panel') as HTMLElement
	if (panel) {
		const rect = panel.getBoundingClientRect()
		dragOffset.value = { x: e.clientX - rect.left, y: e.clientY - rect.top }
	}
	document.addEventListener('mousemove', onMouseMove)
	document.addEventListener('mouseup', onMouseUp)
}

function onMouseMove(e: MouseEvent) {
	if (!isDragging.value) return
	panelPos.value = {
		x: Math.max(0, Math.min(e.clientX - dragOffset.value.x, window.innerWidth - 100)),
		y: Math.max(0, Math.min(e.clientY - dragOffset.value.y, window.innerHeight - 60)),
	}
}

function onMouseUp() {
	isDragging.value = false
	document.removeEventListener('mousemove', onMouseMove)
	document.removeEventListener('mouseup', onMouseUp)
}

// Resize handling
function onResizeMouseDown(e: MouseEvent) {
	e.preventDefault()
	e.stopPropagation()
	isResizing.value = true
	resizeStart.value = {
		x: e.clientX,
		y: e.clientY,
		w: panelWidth.value,
		h: panelHeight.value,
	}
	document.addEventListener('mousemove', onResizeMove)
	document.addEventListener('mouseup', onResizeUp)
}

function onResizeMove(e: MouseEvent) {
	if (!isResizing.value) return
	const dx = e.clientX - resizeStart.value.x
	const dy = e.clientY - resizeStart.value.y
	panelWidth.value = Math.max(320, Math.min(600, resizeStart.value.w + dx))
	panelHeight.value = Math.max(280, Math.min(700, resizeStart.value.h + dy))
}

function onResizeUp() {
	isResizing.value = false
	document.removeEventListener('mousemove', onResizeMove)
	document.removeEventListener('mouseup', onResizeUp)
}

onBeforeUnmount(() => {
	clearHighlight()
	document.removeEventListener('mousemove', onMouseMove)
	document.removeEventListener('mouseup', onMouseUp)
	document.removeEventListener('mousemove', onResizeMove)
	document.removeEventListener('mouseup', onResizeUp)
})

const panelStyle = computed(() => {
	const base: Record<string, string> = {
		width: minimized.value ? 'auto' : `${panelWidth.value}px`,
	}
	if (panelPos.value.x >= 0 && panelPos.value.y >= 0) {
		base.left = `${panelPos.value.x}px`
		base.top = `${panelPos.value.y}px`
		base.right = 'auto'
		base.bottom = 'auto'
	} else {
		base.right = '20px'
		base.bottom = '20px'
	}
	return base
})

const listMaxHeight = computed(() => `${panelHeight.value - 120}px`)
</script>

<template>
	<Teleport to="body">
		<Transition
			enter-active-class="transition-all duration-200 ease-out"
			enter-from-class="opacity-0 translate-y-3 scale-95"
			enter-to-class="opacity-100 translate-y-0 scale-100"
			leave-active-class="transition-all duration-150 ease-in"
			leave-from-class="opacity-100 translate-y-0 scale-100"
			leave-to-class="opacity-0 translate-y-3 scale-95"
		>
			<div
				v-if="debugContext?.panelOpen.value"
				tabindex="-1"
				class="i18n-debug-panel fixed z-[9998] flex flex-col overflow-hidden rounded-xl border-2 border-solid border-surface-5 bg-surface-2 shadow-2xl outline-none"
				:class="{
					'cursor-grabbing': isDragging,
					'select-none': isDragging || isResizing,
				}"
				:style="panelStyle"
				@keydown="onPanelKeydown"
			>
				<!-- Resize handle (bottom-right corner) -->
				<div
					v-if="!minimized"
					class="absolute -bottom-0.5 -right-0.5 z-10 h-4 w-4 cursor-se-resize"
					@mousedown="onResizeMouseDown"
				>
					<svg
						width="10"
						height="10"
						viewBox="0 0 10 10"
						class="absolute bottom-1 right-1 text-secondary/40"
					>
						<circle cx="8.5" cy="8.5" r="1" fill="currentColor" />
						<circle cx="5" cy="8.5" r="1" fill="currentColor" />
						<circle cx="8.5" cy="5" r="1" fill="currentColor" />
					</svg>
				</div>

				<!-- Header -->
				<div
					class="flex items-center gap-2.5 px-3.5 py-2.5 cursor-move select-none border-b border-surface-5/50"
					@mousedown="onHeaderMouseDown"
				>
					<!-- Title group -->
					<div class="flex items-center gap-2">
						<div class="flex h-6 w-6 items-center justify-center rounded-md bg-brand/10">
							<ScanEyeIcon class="h-3.5 w-3.5 text-brand" />
						</div>
						<span class="text-[13px] font-semibold tracking-tight text-primary">
							i18n Inspector
						</span>
					</div>

					<!-- Key count badge -->
					<div class="flex items-center gap-1 rounded-full bg-surface-5/50 px-2 py-0.5">
						<span class="text-[11px] font-medium tabular-nums text-secondary">
							{{ keyCount }} {{ keyCount === 1 ? 'key' : 'keys' }}
						</span>
					</div>

					<!-- Toolbar -->
					<div class="ml-auto flex items-center gap-0.5">
						<ButtonStyled circular type="transparent">
							<button
								v-tooltip="
									debugContext?.keyReveal.value ? 'Hide keys inline' : 'Reveal keys inline'
								"
								@click="toggleKeyReveal"
							>
								<component :is="debugContext?.keyReveal.value ? EyeOffIcon : EyeIcon" />
							</button>
						</ButtonStyled>
						<ButtonStyled circular type="transparent">
							<button v-tooltip="'Toggle CSS debug overlay'" @click="toggleOverlay">
								<ScanEyeIcon />
							</button>
						</ButtonStyled>

						<div class="mx-0.5 h-4 w-px bg-surface-5/60" />

						<ButtonStyled circular type="transparent">
							<button
								v-tooltip="minimized ? 'Expand panel' : 'Minimize panel'"
								@click="minimized = !minimized"
							>
								<component :is="minimized ? MaximizeIcon : MinusIcon" />
							</button>
						</ButtonStyled>
						<ButtonStyled circular type="transparent">
							<button v-tooltip="'Close inspector'" @click="closePanel">
								<XIcon />
							</button>
						</ButtonStyled>
					</div>
				</div>

				<!-- Body (hidden when minimized) -->
				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-[600px]"
					leave-active-class="transition-all duration-150 ease-in"
					leave-from-class="opacity-100 max-h-[600px]"
					leave-to-class="opacity-0 max-h-0"
				>
					<div v-if="!minimized" class="flex flex-col overflow-hidden">
						<!-- Search -->
						<div class="px-3 py-2.5">
							<div
								class="flex items-center gap-2 rounded-lg border border-surface-5/50 bg-surface-1/60 px-3 py-2 transition-all focus-within:border-brand/40 focus-within:bg-surface-1 focus-within:ring-1 focus-within:ring-brand/20"
							>
								<SearchIcon class="h-3.5 w-3.5 shrink-0 text-secondary" />
								<input
									ref="searchInputRef"
									v-model="searchQuery"
									type="text"
									placeholder="Search keys or values..."
									class="w-full !bg-transparent !shadow-none text-[13px] text-primary outline-none placeholder:text-secondary/60"
								/>
								<!-- Result count while searching -->
								<Transition
									enter-active-class="transition-opacity duration-100"
									enter-from-class="opacity-0"
									enter-to-class="opacity-100"
								>
									<span v-if="searchQuery" class="shrink-0 text-[11px] tabular-nums text-secondary">
										{{ matchCount }}
									</span>
								</Transition>
							</div>
						</div>

						<!-- Entry list -->
						<div
							ref="listContainerRef"
							class="overflow-y-auto overscroll-contain scroll-smooth"
							:style="{ maxHeight: listMaxHeight }"
						>
							<TransitionGroup
								move-class="transition-transform duration-200"
								enter-active-class="transition-all duration-150 ease-out"
								enter-from-class="opacity-0 -translate-x-2"
								enter-to-class="opacity-100 translate-x-0"
								leave-active-class="transition-all duration-100 ease-in absolute w-full"
								leave-from-class="opacity-100"
								leave-to-class="opacity-0"
							>
								<div
									v-for="(entry, index) in filteredEntries"
									:key="entry.key"
									class="group relative flex items-center gap-2.5 px-3.5 py-2 transition-colors cursor-pointer"
									:class="[activeEntryIndex === index ? 'bg-brand/8' : 'hover:bg-surface-5/40']"
									:data-active="activeEntryIndex === index"
									@mouseenter="
										() => {
											highlightElement(entry.key)
											activeEntryIndex = index
										}
									"
									@mouseleave="clearHighlight"
									@click="copyKey(entry.key)"
								>
									<!-- Active indicator -->
									<div
										v-if="activeEntryIndex === index"
										class="absolute left-0 top-1/2 h-5 w-[3px] -translate-y-1/2 rounded-r-full bg-brand transition-all"
									/>

									<!-- Entry content -->
									<div class="min-w-0 flex-1">
										<div
											class="font-mono text-[12px] leading-relaxed text-primary truncate"
											:title="entry.key"
											v-html="highlightMatch(entry.key, searchQuery)"
										/>
										<div
											class="mt-0.5 text-[11px] leading-relaxed text-secondary truncate"
											:title="entry.value"
											v-html="highlightMatch(truncate(entry.value, 50), searchQuery)"
										/>
									</div>

									<!-- Actions -->
									<div class="flex shrink-0 items-center gap-1">
										<!-- Copied feedback -->
										<Transition
											enter-active-class="transition-all duration-150 ease-out"
											enter-from-class="opacity-0 scale-90"
											enter-to-class="opacity-100 scale-100"
											leave-active-class="transition-all duration-100"
											leave-from-class="opacity-100"
											leave-to-class="opacity-0 scale-90"
										>
											<span
												v-if="copiedKey === entry.key"
												class="flex items-center gap-1 rounded-md bg-green/10 px-1.5 py-0.5 text-[10px] font-medium text-green"
											>
												<svg width="10" height="10" viewBox="0 0 16 16" fill="none">
													<path
														d="M3 8.5L6.5 12L13 4"
														stroke="currentColor"
														stroke-width="2"
														stroke-linecap="round"
														stroke-linejoin="round"
													/>
												</svg>
												Copied
											</span>
										</Transition>

										<!-- Copy hint (shown on hover when not copied) -->
										<span
											v-if="copiedKey !== entry.key"
											class="text-[10px] text-secondary/0 transition-colors group-hover:text-secondary/60"
										>
											click to copy
										</span>
									</div>
								</div>
							</TransitionGroup>

							<!-- Empty state -->
							<div
								v-if="filteredEntries.length === 0"
								class="flex flex-col items-center justify-center px-4 py-10"
							>
								<div class="flex h-10 w-10 items-center justify-center rounded-xl bg-surface-5/40">
									<SearchIcon class="h-4 w-4 text-secondary/60" />
								</div>
								<p class="mt-3 text-[13px] font-medium text-primary">
									{{ searchQuery ? 'No matches found' : 'No keys registered' }}
								</p>
								<p class="mt-1 text-[11px] text-secondary">
									{{
										searchQuery
											? 'Try a different search term'
											: 'Navigate the app to discover i18n keys'
									}}
								</p>
							</div>
						</div>

						<!-- Footer status bar -->
						<div class="flex items-center justify-between border-t border-surface-5/50 px-3.5 py-2">
							<div class="flex items-center gap-2">
								<div class="h-1.5 w-1.5 rounded-full bg-green animate-pulse" />
								<span class="text-[11px] text-secondary"> Watching </span>
							</div>
							<div class="flex items-center gap-3">
								<span class="text-[10px] text-secondary/60">
									<kbd
										class="rounded border border-surface-5/40 bg-surface-3/60 px-1 py-px text-[10px]"
										>&uarr;</kbd
									>
									<kbd
										class="rounded border border-surface-5/40 bg-surface-3/60 px-1 py-px text-[10px]"
										>&darr;</kbd
									>
									navigate
								</span>
								<span class="text-[10px] text-secondary/60">
									<kbd
										class="rounded border border-surface-5/40 bg-surface-3/60 px-1 py-px text-[10px]"
										>&crarr;</kbd
									>
									copy
								</span>
							</div>
						</div>
					</div>
				</Transition>
			</div>
		</Transition>
	</Teleport>
</template>

<style scoped>
.i18n-debug-panel {
	font-feature-settings: 'cv02', 'cv03', 'cv04', 'cv11';
	-webkit-font-smoothing: antialiased;
	box-shadow:
		0 0 0 1px rgba(0, 0, 0, 0.03),
		0 2px 4px rgba(0, 0, 0, 0.04),
		0 12px 24px rgba(0, 0, 0, 0.12),
		0 24px 48px rgba(0, 0, 0, 0.06);
}

/* Custom scrollbar */
.i18n-debug-panel ::-webkit-scrollbar {
	width: 6px;
}

.i18n-debug-panel ::-webkit-scrollbar-track {
	background: transparent;
}

.i18n-debug-panel ::-webkit-scrollbar-thumb {
	background: var(--surface-5);
	border-radius: 3px;
}

.i18n-debug-panel ::-webkit-scrollbar-thumb:hover {
	background: var(--color-text-tertiary);
}

/* Animate the pulse indicator */
@keyframes soft-pulse {
	0%,
	100% {
		opacity: 1;
	}
	50% {
		opacity: 0.4;
	}
}

.animate-pulse {
	animation: soft-pulse 2s ease-in-out infinite;
}
</style>
