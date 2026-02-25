import type { Component, ComputedRef, Ref } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import type { ComboboxOption } from '../components/base/Combobox.vue'
import { createContext } from '.'

type AutoLinkTarget = string | RouteLocationRaw | (() => void)

export interface InstallationSettingsModpackInfo {
	title: string
	iconUrl?: string
	projectLink?: AutoLinkTarget
	versionName?: string
	versionLink?: AutoLinkTarget
	owner?: {
		name: string
		avatarUrl?: string
		type?: 'user' | 'organization'
		link?: AutoLinkTarget
	}
}

export interface InstallationInfoRow {
	label: string
	value: string
}

export interface InstallationSettingsLinkedAction {
	label: string
	icon: Component
	color: 'standard' | 'red' | 'orange'
	disabled?: boolean
	loading?: boolean
	loadingLabel?: string
	tooltip?: string | null
	handler: () => void
}

export interface InstallationSettingsContext {
	// Linked state
	/**
	 * Controls which UI branch is rendered: when true, the linked modpack info
	 * and action buttons are shown; when false, the platform/version selectors
	 * (unlinked fields below) are shown instead.
	 */
	isLinked: Ref<boolean> | ComputedRef<boolean>
	modpack:
		| Ref<InstallationSettingsModpackInfo | null>
		| ComputedRef<InstallationSettingsModpackInfo | null>
	installationInfo: Ref<InstallationInfoRow[]> | ComputedRef<InstallationInfoRow[]>
	isBusy: Ref<boolean> | ComputedRef<boolean>

	// Linked actions (Change version + Unlink are built-in)
	changeVersion: () => void
	unlink: () => void
	extraLinkedActions?:
		| Ref<InstallationSettingsLinkedAction[]>
		| ComputedRef<InstallationSettingsLinkedAction[]>

	// Unlinked state — only accessed when isLinked is false
	platforms?: Ref<string[]> | ComputedRef<string[]>
	selectedPlatform?: Ref<string>
	gameVersionOptions?: Ref<ComboboxOption<string>[]> | ComputedRef<ComboboxOption<string>[]>
	selectedGameVersion?: Ref<string>
	loaderVersionOptions?: Ref<ComboboxOption<number>[]> | ComputedRef<ComboboxOption<number>[]>
	selectedLoaderVersion?: Ref<number>
	loaderVersionDisplayValue?: Ref<string> | ComputedRef<string>
	formattedLoaderName?: Ref<string> | ComputedRef<string>
	hasChanges?: Ref<boolean> | ComputedRef<boolean>
	isValid?: Ref<boolean> | ComputedRef<boolean>
	isSaving?: Ref<boolean>
	save?: () => Promise<void>

	// Optional
	showSnapshots?: Ref<boolean>
	hasSnapshots?: Ref<boolean> | ComputedRef<boolean>
}

export const [injectInstallationSettings, provideInstallationSettings] =
	createContext<InstallationSettingsContext>(
		'InstallationSettingsLayout',
		'installationSettingsContext',
	)
