import type { RouteLocationRaw } from 'vue-router'

export interface InstallationInfoRow {
	label: string
	value: string
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
	link: string | RouteLocationRaw
	versionNumber?: string
	owner?: InstallationModpackOwner
}

export interface GameVersionOption {
	value: string
	label: string
}

export interface LoaderVersionEntry {
	id: string
	stable?: boolean
}
