<script setup lang="ts">
import { computed, nextTick, ref, useId, watch } from 'vue'

import type { AnchoredTeleportAnchor } from '../../../utils/use-anchored-teleport'
import { pointAnchor, useAnchoredTeleport } from '../../../utils/use-anchored-teleport'
import {
	isDivider,
	isHeading,
	isMenuRow,
	isSubmenu,
	useMenuKeyboard,
	visibleOptions,
} from './overflow-menu'
import OverflowMenuItem from './OverflowMenuItem.vue'
import OverflowMenuPanel from './OverflowMenuPanel.vue'
import OverflowMenuSubmenu from './OverflowMenuSubmenu.vue'
import type { OverflowMenuAction, OverflowMenuLink, OverflowMenuOption } from './types'

const props = defineProps<{
	label: string
}>()

const emit = defineEmits<{
	select: [option: OverflowMenuAction | OverflowMenuLink]
	open: []
	close: []
}>()

const anchor = ref<AnchoredTeleportAnchor | null>(null)
const panel = ref<InstanceType<typeof OverflowMenuPanel> | null>(null)
const panelElement = computed(() => panel.value?.element ?? null)
const placement = ref('bottom-start' as const)
const distance = ref(0)
const menuId = `context-menu-${useId()}`
const currentOptions = ref<OverflowMenuOption[]>([])

const options = computed(() => visibleOptions(currentOptions.value))
const rows = computed(() => options.value.filter(isMenuRow))

const { isOpen, panelStyle, expandOrigin, open, close, updatePosition } = useAnchoredTeleport(
	anchor,
	panelElement,
	placement,
	distance,
)

const { focusedIndex, getItems, handleKeydown, reset } = useMenuKeyboard({
	panel: panelElement,
	rows: () => rows.value,
	onEscape: () => closeMenu(),
	onTab: () => closeMenu(),
})

async function openMenu(event: MouseEvent, menuOptions: OverflowMenuOption[]) {
	currentOptions.value = menuOptions
	anchor.value = pointAnchor(event.clientX, event.clientY)

	if (isOpen.value) {
		await nextTick()
		updatePosition()
		return
	}

	await open()
	emit('open')
	// focus the panel so keys work without highlighting a row
	await nextTick()
	panelElement.value?.focus()
	window.getSelection()?.removeAllRanges()
}

function closeMenu() {
	if (!isOpen.value) return
	reset()
	close()
}

function handleSelect(option: OverflowMenuAction | OverflowMenuLink) {
	emit('select', option)
	if (!option.remainOpen) closeMenu()
}

watch(isOpen, (openState, previousOpenState) => {
	if (!openState && previousOpenState) emit('close')
})

defineExpose({ open: openMenu, close: closeMenu })
</script>

<template>
	<Teleport to="body">
		<OverflowMenuPanel
			ref="panel"
			:open="isOpen"
			:panel-id="menuId"
			:label="props.label"
			:panel-style="panelStyle"
			:origin="expandOrigin"
			tabindex="-1"
			class="focus-visible:outline-none"
			@keydown="handleKeydown"
		>
			<template v-for="(option, index) in options" :key="option.id ?? `${option.type}-${index}`">
				<div v-if="isDivider(option)" role="separator" class="my-1 h-px bg-surface-5" />

				<div
					v-else-if="isHeading(option)"
					class="px-3 pb-1 pt-2 text-xs font-bold uppercase tracking-wide text-secondary first:pt-1"
				>
					{{ option.label }}
				</div>

				<OverflowMenuSubmenu v-else-if="isSubmenu(option)" :option="option" @select="handleSelect">
					<template #trigger>
						<slot :name="option.id" :option="option">
							<component :is="option.icon" v-if="option.icon" aria-hidden="true" />
							{{ option.label }}
						</slot>
					</template>
					<template #item="{ option: child }">
						<slot :name="child.id" :option="child">
							<component :is="child.icon" v-if="child.icon" aria-hidden="true" />
							{{ child.label }}
						</slot>
					</template>
				</OverflowMenuSubmenu>

				<OverflowMenuItem
					v-else
					:option="option"
					@select="handleSelect"
					@focus="focusedIndex = getItems().indexOf($event)"
				>
					<slot :name="option.id" :option="option">
						<component :is="option.icon" v-if="option.icon" aria-hidden="true" />
						{{ option.label }}
					</slot>
				</OverflowMenuItem>
			</template>
		</OverflowMenuPanel>
	</Teleport>
</template>
