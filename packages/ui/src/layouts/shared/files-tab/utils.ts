import {
	BoxIcon,
	BracesIcon,
	FolderCogIcon,
	FolderOpenIcon,
	GlassesIcon,
	GlobeIcon,
	PaintbrushIcon,
	ImagesIcon,
	SquareTextIcon, ClipboardListIcon
} from '@modrinth/assets'
import type { Component } from 'vue'

import type { FileInfo } from '#ui/layouts/shared/files-tab/providers/file-manager.ts'
import { getFileExtensionIcon } from '#ui/utils/auto-icons'
import { getFileExtension } from '#ui/utils/file-extensions'

export interface FileIconStyle {
	icon: Component
	/** Text color class, picked from the theme's primary colors. */
	color: string
}

const DEFAULT_DIRECTORY_ICON: FileIconStyle = { icon: FolderOpenIcon, color: 'text-amber-400' }

const DIRECTORY_ICON_STYLES: Record<string, FileIconStyle> = {
	config: { icon: FolderCogIcon, color: 'text-purple' },
	'crash-reports': { icon: ClipboardListIcon, color: 'text-red-400' },
	logs: { icon: SquareTextIcon, color: 'text-red-400'},
	world: { icon: GlobeIcon, color: 'text-blue' },
	saves: { icon: GlobeIcon, color: 'text-blue' },
	datapacks: { icon: BracesIcon, color: 'text-brand' },
	mods: { icon: BoxIcon, color: 'text-brand' },
	resourcepacks: { icon: PaintbrushIcon, color: 'text-brand' },
	shaderpacks: { icon: GlassesIcon, color: 'text-brand' },
	screenshots: { icon: ImagesIcon, color: 'text-blue' },
}

/** Icon and color for a file or directory, shared by the listing, sidebar tree and tabs. */
export function fileIconFor(file: Pick<FileInfo, 'name' | 'type'>): FileIconStyle {
	if (file.type === 'directory') {
		return DIRECTORY_ICON_STYLES[file.name.toLowerCase()] ?? DEFAULT_DIRECTORY_ICON
	}
	return { icon: getFileExtensionIcon(getFileExtension(file.name)), color: 'text-mist-200' }
}

export function joinDisplayPath(basePath: string | undefined, itemPath: string) {
	if (!basePath) return itemPath

	const separator = basePath.includes('\\') ? '\\' : '/'
	const path = itemPath.replace(/^[\\/]+/, '').replace(/[\\/]+/g, separator)
	const base = basePath.replace(/[\\/]+$/, '')

	return path ? `${base}${separator}${path}` : basePath
}

export function parentInfoFrom(target: string | FileInfo, end?: number) {
	return infoFrom(parentDirectory(typeof target === 'string' ? target : target.path, end))
}

export function infoFrom<BL extends boolean = false>(
	target: string | FileInfo,
	isFile?: BL,
): FileInfo<BL extends true ? 'file' : 'directory'> {
	const path = normalizeDirectoryPath(typeof target == 'string' ? target : target.path)
	const pathParts = path.split('/')
	const name = pathParts[pathParts.length - 1] ?? '/'
	return {
		type: typeof target != 'string' ? target.type : isFile ? 'file' : 'directory',
		path: path,
		name: name,
	} as FileInfo<BL extends true ? 'file' : 'directory'>
}

export function normalizeDirectoryPath(path: string) {
	return `/${path.split('/').filter(Boolean).join('/')}`
}

export function parentDirectory(path: string, end?: number) {
	return `/${path
		.split('/')
		.filter(Boolean)
		.slice(0, end ?? -1)
		.join('/')}`
}

export function isSameInfo(a: FileInfo, b: FileInfo) {
	return a.type === b.type && a.path === b.path
}

/** Path of the entry called `name` inside `directory`. */
export function childPath(directory: string, name: string) {
	return normalizeDirectoryPath(`${directory}/${name}`)
}

/** Whether `path` is `ancestor` itself or lies anywhere beneath it. */
export function isWithinPath(path: string, ancestor: string) {
	const normalizedPath = normalizeDirectoryPath(path)
	const normalizedAncestor = normalizeDirectoryPath(ancestor)
	if (normalizedAncestor === '/') return true
	return (
		normalizedPath === normalizedAncestor || normalizedPath.startsWith(`${normalizedAncestor}/`)
	)
}

/**
 * Rewrites `location` to follow `from` being moved or renamed to `to`. Locations outside of
 * `from` are returned untouched.
 */
export function relocateInfo(location: FileInfo, from: string, to: string): FileInfo {
	if (!isWithinPath(location.path, from)) return location
	const suffix = normalizeDirectoryPath(location.path).slice(normalizeDirectoryPath(from).length)
	return infoFrom({ ...location, path: `${normalizeDirectoryPath(to)}${suffix}` })
}
