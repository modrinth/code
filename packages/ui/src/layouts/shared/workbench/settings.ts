import {
	defineSetting,
	type SettingDefinitionBase,
	type SettingGroup,
	type SettingsStore,
} from '#ui/components/settings'
import { defineMessages } from '#ui/composables/i18n'

import type { WorkbenchContext } from './providers/workbench'
import type { WorkbenchVariant } from './types'

const messages = defineMessages({
	group: { id: 'workbench.settings.group', defaultMessage: 'Layout' },
	groupDescription: {
		id: 'workbench.settings.group-description',
		defaultMessage: 'How the panels around the workspace look and behave.',
	},
	variant: { id: 'workbench.settings.variant', defaultMessage: 'Panel style' },
	variantDescription: {
		id: 'workbench.settings.variant-description',
		defaultMessage: 'Square panels sit edge to edge, while rounded panels float apart as cards.',
	},
	square: { id: 'workbench.settings.variant.square', defaultMessage: 'Square' },
	rounded: { id: 'workbench.settings.variant.rounded', defaultMessage: 'Rounded' },
	toggles: { id: 'workbench.settings.toggles', defaultMessage: 'Panel toggles in the tab bar' },
	togglesDescription: {
		id: 'workbench.settings.toggles-description',
		defaultMessage:
			'Show buttons for the left and right panels at the ends of the tab bar. They can always be shown and hidden from the layout menu.',
	},
	popouts: { id: 'workbench.settings.popouts', defaultMessage: 'Pop out panels' },
	popoutsDescription: {
		id: 'workbench.settings.popouts-description',
		defaultMessage: 'Let panels be moved into windows of their own from their menus.',
	},
})

/** A workbench's own settings, kept with its layout rather than in a settings store. */
export const workbenchSettings = {
	variant: defineSetting.enum<WorkbenchVariant>({
		id: 'workbench-variant',
		title: messages.variant,
		description: messages.variantDescription,
		default: 'square',
		display: 'chips',
		entries: [
			{ value: 'square', label: messages.square },
			{ value: 'rounded', label: messages.rounded },
		],
	}),
	toggles: defineSetting.toggle({
		id: 'workbench-corner-toggles',
		title: messages.toggles,
		description: messages.togglesDescription,
		default: false,
	}),
	popouts: defineSetting.toggle({
		id: 'workbench-popouts',
		title: messages.popouts,
		description: messages.popoutsDescription,
		default: true,
	}),
}

/** The workbench's settings, as offered in a host's settings (e.g. its settings modal). */
export function workbenchSettingGroup(workbench: WorkbenchContext): SettingGroup {
	return {
		id: 'workbench',
		title: messages.group,
		description: messages.groupDescription,
		settings: [
			workbenchSettings.variant,
			...(workbench.mode.toolbar ? [workbenchSettings.toggles] : []),
			...(workbench.canUsePopouts ? [workbenchSettings.popouts] : []),
		],
	}
}

/** Reads and writes the workbench's settings on the workbench itself, which saves them with its layout. */
export function workbenchSettingsStore(workbench: WorkbenchContext): SettingsStore {
	return {
		get<T>(definition: SettingDefinitionBase<T>): T {
			if (definition.id === workbenchSettings.variant.id) return workbench.variant.value as T
			if (definition.id === workbenchSettings.popouts.id) return workbench.popoutsEnabled.value as T
			if (definition.id === workbenchSettings.toggles.id) {
				return (workbench.toggleLocation.value === 'tabs') as T
			}
			return definition.default
		},
		set<T>(definition: SettingDefinitionBase<T>, value: T) {
			if (definition.id === workbenchSettings.variant.id) {
				workbench.setVariant(value as WorkbenchVariant)
			} else if (definition.id === workbenchSettings.popouts.id) {
				workbench.setPopoutsEnabled(value as boolean)
			} else if (definition.id === workbenchSettings.toggles.id) {
				workbench.setToggleLocation(value ? 'tabs' : 'menu')
			}
		},
	}
}
