<template>
	<li
		role="button"
		:class="[
			containerClasses,
			isDragOver && type === 'directory' ? 'bg-brand-highlight' : '',
			isDragging ? 'opacity-50' : '',
		]"
		tabindex="0"
		draggable="true"
		@click="selectItem"
		@contextmenu="openContextMenu"
		@keydown="(e) => e.key === 'Enter' && selectItem()"
		@mouseenter="handleMouseEnter"
		@dragstart="handleDragStart"
		@dragend="handleDragEnd"
		@dragenter.prevent="handleDragEnter"
		@dragover.prevent="handleDragOver"
		@dragleave.prevent="handleDragLeave"
		@drop.prevent="handleDrop"
	>
		<div class="pointer-events-none flex flex-1 items-center gap-3 truncate">
			<Checkbox
				class="pointer-events-auto"
				:model-value="selected"
				@click.stop
				@update:model-value="emit('toggle-select')"
			/>
			<div class="pointer-events-none flex size-5 items-center justify-center">
				<component :is="iconComponent" class="size-5" />
			</div>
			<div class="pointer-events-none flex flex-col truncate">
				<span
					class="pointer-events-none truncate group-hover:text-contrast group-focus:text-contrast"
				>
					{{ name }}
				</span>
			</div>
		</div>
		<div class="pointer-events-auto flex w-fit flex-shrink-0 items-center gap-4 md:gap-12">
			<span class="hidden w-[100px] text-nowrap text-sm text-secondary md:block">
				{{ formattedSize }}
			</span>
			<span class="hidden w-[160px] text-nowrap text-sm text-secondary md:block">
				{{ formattedCreationDate }}
			</span>
			<span class="hidden w-[160px] text-nowrap text-sm text-secondary md:block">
				{{ formattedModifiedDate }}
			</span>
			<ButtonStyled circular type="transparent">
				<TeleportOverflowMenu :options="menuOptions">
					<MoreHorizontalIcon class="h-5 w-5 bg-transparent" />
					<template #extract><PackageOpenIcon /> Extract</template>
					<template #rename><EditIcon /> Rename</template>
					<template #move><RightArrowIcon /> Move</template>
					<template #download><DownloadIcon /> Download</template>
					<template #delete><TrashIcon /> Delete</template>
				</TeleportOverflowMenu>
			</ButtonStyled>
		</div>
	</li>
</template>

<script setup lang="ts">
import {
	DownloadIcon,
	EditIcon,
	FolderCogIcon,
	FolderOpenIcon,
	GlobeIcon,
	MoreHorizontalIcon,
	PackageOpenIcon,
	PaletteIcon,
	RightArrowIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	Checkbox,
	getFileExtension,
	getFileExtensionIcon,
	isEditableFile as isEditableFileExt,
	isImageFile,
} from '@modrinth/ui'
import { computed, ref, shallowRef } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import TeleportOverflowMenu from './TeleportOverflowMenu.vue'

interface FileItemProps {
	name: string
	type: 'directory' | 'file'
	size?: number
	count?: number
	modified: number
	created: number
	path: string
	index: number
	isLast: boolean
	selected: boolean
}

const props = defineProps<FileItemProps>()

const emit = defineEmits<{
	rename: [item: { name: string; type: string; path: string }]
	move: [item: { name: string; type: string; path: string }]
	download: [item: { name: string; type: string; path: string }]
	delete: [item: { name: string; type: string; path: string }]
	edit: [item: { name: string; type: string; path: string }]
	extract: [item: { name: string; type: string; path: string }]
	hover: [item: { name: string; type: string; path: string }]
	moveDirectTo: [item: { name: string; type: string; path: string; destination: string }]
	contextmenu: [x: number, y: number]
	'toggle-select': []
}>()

const isDragOver = ref(false)
const isDragging = ref(false)

const units = Object.freeze(['B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB'])

const route = shallowRef(useRoute())
const router = useRouter()

const containerClasses = computed(() => [
	'group m-0 flex w-full select-none items-center justify-between overflow-hidden border-0 border-t border-solid border-surface-3 px-4 py-3 focus:!outline-none',
	props.selected ? 'bg-surface-3' : props.index % 2 === 0 ? 'bg-surface-2' : 'file-row-alt',
	props.isLast ? 'rounded-b-[20px] border-b' : '',
	isEditableFile.value ? 'cursor-pointer' : props.type === 'directory' ? 'cursor-pointer' : '',
	isDragOver.value ? '!bg-brand-highlight' : '',
	'transition-colors duration-100 hover:!bg-surface-4 hover:!brightness-100 focus:!bg-surface-4 focus:!brightness-100',
])

const fileExtension = computed(() => getFileExtension(props.name))

const isZip = computed(() => fileExtension.value === 'zip')

const menuOptions = computed(() => [
	{
		id: 'extract',
		shown: isZip.value,
		action: () => emit('extract', { name: props.name, type: props.type, path: props.path }),
	},
	{
		divider: true,
		shown: isZip.value,
	},
	{
		id: 'rename',
		action: () => emit('rename', { name: props.name, type: props.type, path: props.path }),
	},
	{
		id: 'move',
		action: () => emit('move', { name: props.name, type: props.type, path: props.path }),
	},
	{
		id: 'download',
		action: () => emit('download', { name: props.name, type: props.type, path: props.path }),
		shown: props.type !== 'directory',
	},
	{
		id: 'delete',
		action: () => emit('delete', { name: props.name, type: props.type, path: props.path }),
		color: 'red' as const,
	},
])

const iconComponent = computed(() => {
	if (props.type === 'directory') {
		if (props.name === 'config') return FolderCogIcon
		if (props.name === 'world') return GlobeIcon
		if (props.name === 'resourcepacks') return PaletteIcon
		return FolderOpenIcon
	}

	return getFileExtensionIcon(fileExtension.value)
})

const formattedModifiedDate = computed(() => {
	const date = new Date(props.modified * 1000)
	return `${date.toLocaleDateString('en-US', {
		month: '2-digit',
		day: '2-digit',
		year: '2-digit',
	})}, ${date.toLocaleTimeString('en-US', {
		hour: 'numeric',
		minute: 'numeric',
		hour12: true,
	})}`
})

const formattedCreationDate = computed(() => {
	const date = new Date(props.created * 1000)
	return `${date.toLocaleDateString('en-US', {
		month: '2-digit',
		day: '2-digit',
		year: '2-digit',
	})}, ${date.toLocaleTimeString('en-US', {
		hour: 'numeric',
		minute: 'numeric',
		hour12: true,
	})}`
})

const isEditableFile = computed(() => {
	if (props.type === 'file') {
		const ext = fileExtension.value
		return !props.name.includes('.') || isEditableFileExt(ext) || isImageFile(ext)
	}
	return false
})

const formattedSize = computed(() => {
	if (props.type === 'directory') {
		return `${props.count} ${props.count === 1 ? 'item' : 'items'}`
	}

	if (props.size === undefined) return ''
	const bytes = props.size
	if (bytes === 0) return '0 B'

	const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1)
	const size = (bytes / Math.pow(1024, exponent)).toFixed(2)
	return `${size} ${units[exponent]}`
})

function openContextMenu(event: MouseEvent) {
	event.preventDefault()
	emit('contextmenu', event.clientX, event.clientY)
}

function handleMouseEnter() {
	emit('hover', { name: props.name, type: props.type, path: props.path })
}

function navigateToFolder() {
	const currentPath = route.value.query.path?.toString() || ''
	const newPath = currentPath.endsWith('/')
		? `${currentPath}${props.name}`
		: `${currentPath}/${props.name}`
	router.push({ query: { path: newPath } })
}

const isNavigating = ref(false)

function selectItem() {
	if (isNavigating.value) return
	isNavigating.value = true

	if (props.type === 'directory') {
		navigateToFolder()
	} else if (props.type === 'file' && isEditableFile.value) {
		emit('edit', { name: props.name, type: props.type, path: props.path })
	}

	setTimeout(() => {
		isNavigating.value = false
	}, 500)
}

function handleDragStart(event: DragEvent) {
	if (!event.dataTransfer) return
	isDragging.value = true

	const dragGhost = document.createElement('div')
	dragGhost.className =
		'fixed left-0 top-0 flex items-center max-w-[500px] flex-row gap-3 rounded-lg bg-bg-raised p-3 shadow-lg pointer-events-none'

	const nameSpan = document.createElement('span')
	nameSpan.className = 'font-bold truncate text-contrast'
	nameSpan.textContent = props.name

	dragGhost.appendChild(nameSpan)
	document.body.appendChild(dragGhost)

	event.dataTransfer.setDragImage(dragGhost, 0, 0)

	requestAnimationFrame(() => {
		document.body.removeChild(dragGhost)
	})

	event.dataTransfer.setData(
		'application/modrinth-file-move',
		JSON.stringify({
			name: props.name,
			type: props.type,
			path: props.path,
		}),
	)
	event.dataTransfer.effectAllowed = 'move'
}

function isChildPath(parentPath: string, childPath: string) {
	return childPath.startsWith(parentPath + '/')
}

function handleDragEnd() {
	isDragging.value = false
}

function handleDragEnter() {
	if (props.type !== 'directory') return
	isDragOver.value = true
}

function handleDragOver(event: DragEvent) {
	if (props.type !== 'directory' || !event.dataTransfer) return
	event.dataTransfer.dropEffect = 'move'
}

function handleDragLeave() {
	isDragOver.value = false
}

function handleDrop(event: DragEvent) {
	isDragOver.value = false
	if (props.type !== 'directory' || !event.dataTransfer) return

	try {
		const dragData = JSON.parse(event.dataTransfer.getData('application/modrinth-file-move'))

		if (dragData.path === props.path) return

		if (dragData.type === 'directory' && isChildPath(dragData.path, props.path)) {
			console.error('Cannot move a folder into its own subfolder')
			return
		}

		emit('moveDirectTo', {
			name: dragData.name,
			type: dragData.type,
			path: dragData.path,
			destination: props.path,
		})
	} catch (error) {
		console.error('Error handling file drop:', error)
	}
}
</script>

<style scoped>
.file-row-alt {
	background: color-mix(in srgb, var(--surface-2), black 3%);
}

:global(.dark-mode) .file-row-alt,
:global(.dark) .file-row-alt,
:global(.oled-mode) .file-row-alt {
	background: color-mix(in srgb, var(--surface-2), black 10%);
}
</style>
