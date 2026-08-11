<template>
	<Teleport to="#teleports">
		<Transition name="floating-expand">
			<div
				v-if="visible"
				ref="menuRef"
				class="fixed isolate z-[9999] flex w-fit min-w-[180px] flex-col gap-2 overflow-hidden rounded-2xl border border-solid border-surface-5 bg-bg-raised p-2 shadow-lg"
				:style="{ left: `${position.x}px`, top: `${position.y}px`, transformOrigin: 'top left' }"
				role="menu"
				tabindex="-1"
				@mousedown.stop
			>
				<Button
					type="quiet"
					class="w-full !justify-start !whitespace-nowrap"
					role="menuitem"
					@click="handleCopyFilename"
				>
					<ClipboardCopyIcon class="size-5" />
					{{ formatMessage(commonMessages.copyFilenameButton) }}
				</Button>
				<Button
					type="quiet"
					class="w-full !justify-start !whitespace-nowrap"
					role="menuitem"
					@click="handleCopyPath"
				>
					<ClipboardCopyIcon class="size-5" />
					{{ formatMessage(commonMessages.copyFullPathButton) }}
				</Button>
				<Button
					v-if="ctx.openInFolder"
					type="quiet"
					class="w-full !justify-start !whitespace-nowrap"
					role="menuitem"
					@click="handleOpenInFolder"
				>
					<FolderOpenIcon class="size-5" />
					{{ formatMessage(commonMessages.openInFolderButton) }}
				</Button>
				<div class="h-px w-full bg-surface-5" />
				<template v-for="(option, index) in menuOptions" :key="index">
					<div
						v-if="'divider' in option && option.divider && option.shown !== false"
						class="h-px w-full bg-surface-5"
					/>
					<Button
						v-else-if="'id' in option && option.shown !== false"
						v-tooltip="option.tooltip"
						type="quiet"
						:color="
							option.color && option.color !== 'standard'
								? option.color === 'medal-promo'
									? 'medal_promotion'
									: option.color
								: undefined
						"
						:disabled="option.disabled"
						class="w-full !justify-start !whitespace-nowrap"
						role="menuitem"
						@click="handleOptionClick(option)"
					>
						<slot :name="option.id" />
					</Button>
				</template>
			</div>
		</Transition>
	</Teleport>
</template>

<script setup lang="ts">
import { ClipboardCopyIcon, FolderOpenIcon } from '@modrinth/assets'
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { Button } from '#ui/components/base/buttons'
import { useVIntl } from '#ui/composables/i18n'
import { injectNotificationManager } from '#ui/providers/web-notifications'
import { commonMessages } from '#ui/utils/common-messages'

import { injectFileManager } from '../providers/file-manager'
import type { FileContextMenuOption, FileItem } from '../types'
import { joinDisplayPath } from '../utils'

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const ctx = injectFileManager()

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
	addNotification({ title: formatMessage(commonMessages.copiedFilenameLabel), type: 'success' })
	hide()
}

function getFullPath() {
	if (!currentItem.value) return ''
	return joinDisplayPath(ctx.basePath?.value, currentItem.value.path)
}

function handleCopyPath() {
	if (!currentItem.value) return
	navigator.clipboard.writeText(getFullPath())
	addNotification({ title: formatMessage(commonMessages.copiedPathLabel), type: 'success' })
	hide()
}

function handleOpenInFolder() {
	if (!currentItem.value) return
	ctx.openInFolder?.(getFullPath())
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
