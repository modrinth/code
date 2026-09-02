<template>
	<nav :aria-label="ariaLabel" class="w-full">
		<ul
			class="card-shadow m-0 mb-3 flex list-none flex-col items-start gap-1.5 rounded-2xl border border-solid border-surface-4 bg-surface-3 p-4"
			:class="{ 'pt-3': filteredItems?.[0]?.type === 'heading' }"
		>
			<slot v-if="hasSlotContent" />

			<template v-else>
				<li v-for="(item, idx) in filteredItems" :key="getKey(item, idx)" class="contents">
					<hr v-if="isSeparator(item)" class="my-1 w-full border-t border-solid" />

					<div
						v-else-if="isHeading(item)"
						class="px-4 pb-1 pt-2 text-xs font-bold uppercase tracking-wide text-secondary"
					>
						{{ item.label }}
					</div>

					<ButtonLink
						v-else-if="item.link ?? item.to"
						type="quiet"
						:interaction="isActive(item as NavStackLinkItem) ? 'none' : 'surface'"
						:to="(item.link ?? item.to) as string"
						class="nav-item !h-auto !w-full !justify-start !rounded-xl !px-4 !py-2.5 text-left leading-tight"
						:class="{ 'is-active': isActive(item as NavStackLinkItem) }"
					>
						<component
							:is="item.icon"
							v-if="item.icon"
							aria-hidden="true"
							class="h-5 w-5 shrink-0"
						/>
						<span
							:ref="(element) => setItemLabelRef(getKey(item, idx), element)"
							v-tooltip="itemLabelTooltip(getKey(item, idx), item.label)"
							class="min-w-0 flex-1 truncate text-contrast"
						>
							{{ item.label }}
						</span>
						<span
							v-if="item.badge != null"
							class="shrink-0 rounded-full bg-brand-highlight px-2 text-sm font-bold text-brand"
						>
							{{ String(item.badge) }}
						</span>
						<span v-if="item.chevron" class="ml-auto"><ChevronRightIcon /></span>
					</ButtonLink>

					<Button
						v-else-if="item.action"
						type="quiet"
						interaction="surface"
						:color="item.danger ? 'red' : undefined"
						class="nav-item !h-auto !w-full !justify-start !rounded-xl !px-4 !py-2.5 text-left leading-tight"
						@click="item.action"
					>
						<component
							:is="item.icon"
							v-if="item.icon"
							aria-hidden="true"
							class="h-5 w-5 shrink-0"
						/>
						<span
							:ref="(element) => setItemLabelRef(getKey(item, idx), element)"
							v-tooltip="itemLabelTooltip(getKey(item, idx), item.label)"
							class="min-w-0 flex-1 truncate text-contrast"
						>
							{{ item.label }}
						</span>
						<span
							v-if="item.badge != null"
							class="shrink-0 rounded-full bg-brand-highlight px-2 text-sm font-bold text-brand"
						>
							{{ String(item.badge) }}
						</span>
					</Button>

					<span v-else>You frog. 🐸</span>
				</li>
			</template>
		</ul>
	</nav>
</template>

<script setup lang="ts">
import { ChevronRightIcon } from '@modrinth/assets'
import { Button, ButtonLink, truncatedTooltip } from '@modrinth/ui'
import { type Component, type ComponentPublicInstance, computed, ref, useSlots } from 'vue'

type NavStackBaseItem = {
	label: string
	icon?: Component | string
	badge?: string | number | null
	chevron?: boolean
	danger?: boolean
}

type NavStackLinkItem = NavStackBaseItem & {
	type?: 'item'
	link?: string | null
	to?: string | null
	action?: (() => void) | null
	matchNested?: boolean
}

type NavStackSeparator = { type: 'separator' }
type NavStackHeading = { type: 'heading'; label: string }

export type NavStackEntry = (NavStackLinkItem | NavStackSeparator | NavStackHeading) & {
	shown?: boolean
}

const props = defineProps<{
	items?: NavStackEntry[]
	ariaLabel?: string
}>()

const ariaLabel = computed(() => props.ariaLabel ?? 'Section navigation')

const route = useRoute()

const slots = useSlots()
const hasSlotContent = computed(() => {
	const content = slots.default?.()
	return !!(content && content.length)
})

const itemLabelRefs = ref<Record<string, HTMLElement | null>>({})

function setItemLabelRef(key: string, element: Element | ComponentPublicInstance | null) {
	itemLabelRefs.value[key] = element instanceof HTMLElement ? element : null
}

function itemLabelTooltip(key: string, label: string) {
	return truncatedTooltip(itemLabelRefs.value[key], label)
}

function isSeparator(item: NavStackEntry): item is NavStackSeparator {
	return (item as any).type === 'separator'
}

function isHeading(item: NavStackEntry): item is NavStackHeading {
	return (item as any).type === 'heading'
}

function getKey(item: NavStackEntry, idx: number) {
	if (isSeparator(item)) return `sep-${idx}`
	if (isHeading(item)) return `head-${item.label}-${idx}`
	const link = (item as NavStackLinkItem).link ?? (item as NavStackLinkItem).to
	return link ? `link-${link}` : `action-${(item as NavStackLinkItem).label}-${idx}`
}

function isActive(item: NavStackLinkItem): boolean {
	const link = item.link ?? item.to
	if (!link) return false

	const currentPath = route.path

	if (item.matchNested) {
		return currentPath.startsWith(link)
	}

	return currentPath === link
}

const filteredItems = computed(() => props.items?.filter((x) => x.shown === undefined || x.shown))
</script>

<style lang="scss" scoped>
li {
	text-align: unset;
}
.router-link-exact-active.nav-item,
.nav-item.is-active {
	background: var(--color-button-bg-selected);
	color: var(--color-button-text-selected);
}
.router-link-exact-active.nav-item .text-contrast,
.nav-item.is-active .text-contrast {
	color: var(--color-button-text-selected);
}
</style>
