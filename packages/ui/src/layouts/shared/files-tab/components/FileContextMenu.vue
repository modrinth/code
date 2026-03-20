<template>
	<Teleport to="#teleports">
		<Transition
			enter-active-class="transition duration-125 ease-out"
			enter-from-class="transform scale-75 opacity-0"
			enter-to-class="transform scale-100 opacity-100"
			leave-active-class="transition duration-125 ease-in"
			leave-from-class="transform scale-100 opacity-100"
			leave-to-class="transform scale-75 opacity-0"
		>
			<div
				v-if="visible"
				ref="menuRef"
				class="experimental-styles-within fixed isolate z-[9999] flex w-fit min-w-[180px] flex-col gap-2 overflow-hidden rounded-2xl border border-solid border-surface-5 bg-bg-raised p-2 shadow-lg"
				:style="{ left: `${position.x}px`, top: `${position.y}px` }"
				role="menu"
				tabindex="-1"
				@mousedown.stop
			>
				<ButtonStyled type="transparent">
					<button
						class="w-full !justify-start !whitespace-nowrap"
						role="menuitem"
						@click="handleCopyFilename"
					>
						<ClipboardCopyIcon class="size-5" />
						{{ formatMessage(messages.copyFilename) }}
					</button>
				</ButtonStyled>
				<ButtonStyled type="transparent">
					<button
						class="w-full !justify-start !whitespace-nowrap"
						role="menuitem"
						@click="handleCopyPath"
					>
						<ClipboardCopyIcon class="size-5" />
						{{ formatMessage(messages.copyFullPath) }}
					</button>
				</ButtonStyled>
				<div class="h-px w-full bg-surface-5" />
				<template v-for="(option, index) in menuOptions" :key="index">
					<div
						v-if="'divider' in option && option.divider && option.shown !== false"
						class="h-px w-full bg-surface-5"
					/>
					<ButtonStyled
						v-else-if="'id' in option && option.shown !== false"
						type="transparent"
						:color="option.color"
					>
						<button
							v-tooltip="option.tooltip"
							:disabled="option.disabled"
							class="w-full !justify-start !whitespace-nowrap"
							role="menuitem"
							@click="handleOptionClick(option)"
						>
							<slot :name="option.id" />
						</button>
					</ButtonStyled>
				</template>
			</div>
		</Transition>
	</Teleport>
</template>

<script setup lang="ts">
import { ClipboardCopyIcon } from '@modrinth/assets'
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectNotificationManager } from '#ui/providers/web-notifications'

import type { FileContextMenuOption, FileItem } from '../types'

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()

const messages = defineMessages({
	copyFilename: {
		id: 'files.context-menu.copy-filename',
		defaultMessage: 'Copy filename',
	},
	copyFullPath: {
		id: 'files.context-menu.copy-full-path',
		defaultMessage: 'Copy full path',
	},
	copiedFilename: {
		id: 'files.context-menu.copied-filename',
		defaultMessage: 'Copied filename',
	},
	copiedPath: {
		id: 'files.context-menu.copied-path',
		defaultMessage: 'Copied path',
	},
})

const visible = ref(false)
const menuRef = ref<HTMLElement>()
const position = ref({ x: 0, y: 0 })
const currentItem = ref<FileItem | null>(null)
const menuOptions = ref<FileContextMenuOption[]>([])

function show(item: FileItem, x: number, y: number, options: typeof menuOptions.value) {
	currentItem.value = item
	menuOptions.value = options
	position.value = { x, y }
	visible.value = true

	nextTick(() => {
		if (!menuRef.value) return
		const rect = menuRef.value.getBoundingClientRect()
		const padding = 10
		if (rect.right > window.innerWidth - padding) {
			position.value.x = Math.max(padding, x - rect.width)
		}
		if (rect.bottom > window.innerHeight - padding) {
			position.value.y = Math.max(padding, y - rect.height)
		}
	})
}

function hide() {
	visible.value = false
	currentItem.value = null
}

function handleCopyFilename() {
	if (!currentItem.value) return
	navigator.clipboard.writeText(currentItem.value.name)
	addNotification({ title: formatMessage(messages.copiedFilename), type: 'success' })
	hide()
}

function handleCopyPath() {
	if (!currentItem.value) return
	navigator.clipboard.writeText(currentItem.value.path)
	addNotification({ title: formatMessage(messages.copiedPath), type: 'success' })
	hide()
}

function handleOptionClick(option: { action?: () => void }) {
	option.action?.()
	hide()
}

function onClickOutside(event: MouseEvent) {
	if (menuRef.value && !menuRef.value.contains(event.target as Node)) {
		hide()
	}
}

function onEscape(event: KeyboardEvent) {
	if (event.key === 'Escape') {
		hide()
	}
}

onMounted(() => {
	document.addEventListener('mousedown', onClickOutside)
	document.addEventListener('keydown', onEscape)
})

onBeforeUnmount(() => {
	document.removeEventListener('mousedown', onClickOutside)
	document.removeEventListener('keydown', onEscape)
})

watch(visible, (v) => {
	if (!v) currentItem.value = null
})

defineExpose({ show, hide })
</script>
