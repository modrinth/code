import type { RouteLocationRaw } from 'vue-router'

export interface InstallationInfoRow {
	label: string
	value: string | null
}

export interface InstallationModpackOwner {
	id: string
	name: string
	iconUrl?: string
	type: 'user' | 'organization'
}

export interface InstallationModpackData {
	iconUrl?: string
	title: string
	link?: string | RouteLocationRaw
	versionNumber?: string
	filename?: string
	owner?: InstallationModpackOwner
}

export interface GameVersionOption {
	value: string
	label: string
}

export interface LoaderVersionEntry {
	id: string
	stable?: boolean
	/** Shown in the loader-version combobox when set; defaults to `id` */
	label?: string
	/** Paper build channel for optional UI (e.g. combobox pill); not used by Combobox itself */
	channelTag?: 'ALPHA' | 'BETA'
}

export interface ContentDiffItem {
	type: 'added' | 'removed' | 'updated'
	projectName?: string
	fileName?: string
	currentVersionName?: string
	newVersionName?: string
}

export interface ContentDiffPreview {
	diffs: ContentDiffItem[]
	newGameVersion: string
	newLoaderVersion: string
	hasUnknownContent: boolean
}
