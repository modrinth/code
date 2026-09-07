<script setup lang="ts">
import { computed, nextTick, onMounted, ref, useId, watch } from 'vue'

import type { AnchoredTeleportAnchor } from '../../../utils/use-anchored-teleport'
import { pointAnchor, useAnchoredTeleport } from '../../../utils/use-anchored-teleport'
import {
	isDivider,
	isHeading,
	isMenuRow,
	isSubmenu,
	useMenuKeyboard,
	visibleOptions,
} from './button-menu/button-menu'
import ButtonMenuItem from './button-menu/ButtonMenuItem.vue'
import ButtonMenuPanel from './button-menu/ButtonMenuPanel.vue'
import ButtonMenuSubmenu from './button-menu/ButtonMenuSubmenu.vue'
import type { ButtonMenuAction, ButtonMenuLink, ButtonMenuOption } from './types'

const props = defineProps<{
	label: string
}>()

const emit = defineEmits<{
	select: [option: ButtonMenuAction | ButtonMenuLink]
	open: []
	close: []
}>()

const anchor = ref<AnchoredTeleportAnchor | null>(null)
const panel = ref<InstanceType<typeof ButtonMenuPanel> | null>(null)
const panelElement = computed(() => panel.value?.element ?? null)
const placement = ref('bottom-start' as const)
const distance = ref(0)
const menuId = `context-menu-${useId()}`
const currentOptions = ref<ButtonMenuOption[]>([])

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

async function openMenu(event: MouseEvent, menuOptions: ButtonMenuOption[]) {
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

function handleSelect(option: ButtonMenuAction | ButtonMenuLink) {
	emit('select', option)
	if (!option.remainOpen) closeMenu()
}

watch(isOpen, (openState, previousOpenState) => {
	if (!openState && previousOpenState) emit('close')
})

const isClient = ref(false)
onMounted(() => {
	isClient.value = true
})

defineExpose({ open: openMenu, close: closeMenu })
</script>

<template>
	<Teleport v-if="isClient" to="body">
		<ButtonMenuPanel
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

				<ButtonMenuSubmenu v-else-if="isSubmenu(option)" :option="option" @select="handleSelect">
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
				</ButtonMenuSubmenu>

				<ButtonMenuItem
					v-else
					:option="option"
					@select="handleSelect"
					@focus="focusedIndex = getItems().indexOf($event)"
				>
					<slot :name="option.id" :option="option">
						<component :is="option.icon" v-if="option.icon" aria-hidden="true" />
						{{ option.label }}
					</slot>
				</ButtonMenuItem>
			</template>
		</ButtonMenuPanel>
	</Teleport>
</template>
