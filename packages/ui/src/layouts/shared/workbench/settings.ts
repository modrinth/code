import {
	defineSetting,
	type SettingDefinitionBase,
	type SettingGroup,
	type SettingsStore,
} from '#ui/components/settings'
import { defineMessages } from '#ui/composables/i18n'

import type { WorkbenchContext } from './providers/workbench'
import type { WorkbenchPopoutRestore, WorkbenchToggleAction, WorkbenchVariant } from './types'

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
			'Show buttons for the left and right panels at the ends of the tab bar, and for the bottom panel in its own. They can always be shown and hidden from the layout menu.',
	},
	loneTabs: { id: 'workbench.settings.lone-tabs', defaultMessage: 'Hide lone tab bars' },
	loneTabsDescription: {
		id: 'workbench.settings.lone-tabs-description',
		defaultMessage:
			'Hide the tab bar of panels on their own outside the center, giving them its room. A handle at their top right shows it again. Any tab bar can also be hidden from its menu.',
	},
	toggleAction: { id: 'workbench.settings.toggle-action', defaultMessage: 'Panel toggles' },
	toggleActionDescription: {
		id: 'workbench.settings.toggle-action-description',
		defaultMessage:
			'Whether the buttons for the left, right and bottom panels hide them entirely, or collapse them to a strip of their tabs.',
	},
	toggleActionHide: { id: 'workbench.settings.toggle-action.hide', defaultMessage: 'Hide' },
	toggleActionCollapse: {
		id: 'workbench.settings.toggle-action.collapse',
		defaultMessage: 'Collapse',
	},
	popouts: { id: 'workbench.settings.popouts', defaultMessage: 'Pop out panels' },
	popoutRestore: {
		id: 'workbench.settings.popout-restore',
		defaultMessage: 'Popped out panels on reopening',
	},
	popoutRestoreDescription: {
		id: 'workbench.settings.popout-restore-description',
		defaultMessage:
			'When the page is opened again: return popped out panels to the main window, return them while offering to reopen their windows, or reopen their windows (on your next click, if the browser asks for one).',
	},
	popoutRestoreClose: {
		id: 'workbench.settings.popout-restore.close',
		defaultMessage: 'Close and merge',
	},
	popoutRestorePrompt: {
		id: 'workbench.settings.popout-restore.prompt',
		defaultMessage: 'Merge and ask',
	},
	popoutRestorePreserve: {
		id: 'workbench.settings.popout-restore.preserve',
		defaultMessage: 'Preserve',
	},
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
	toggleAction: defineSetting.enum<WorkbenchToggleAction>({
		id: 'workbench-toggle-action',
		title: messages.toggleAction,
		description: messages.toggleActionDescription,
		default: 'hide',
		display: 'chips',
		entries: [
			{ value: 'hide', label: messages.toggleActionHide },
			{ value: 'collapse', label: messages.toggleActionCollapse },
		],
	}),
	loneTabs: defineSetting.toggle({
		id: 'workbench-lone-tabs',
		title: messages.loneTabs,
		description: messages.loneTabsDescription,
		default: false,
	}),
	popouts: defineSetting.toggle({
		id: 'workbench-popouts',
		title: messages.popouts,
		description: messages.popoutsDescription,
		default: true,
	}),
	popoutRestore: defineSetting.enum<WorkbenchPopoutRestore>({
		id: 'workbench-popout-restore',
		title: messages.popoutRestore,
		description: messages.popoutRestoreDescription,
		default: 'prompt',
		display: 'chips',
		entries: [
			{ value: 'close', label: messages.popoutRestoreClose },
			{ value: 'prompt', label: messages.popoutRestorePrompt },
			{ value: 'preserve', label: messages.popoutRestorePreserve },
		],
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
			workbenchSettings.loneTabs,
			...(workbench.mode.toolbar ? [workbenchSettings.toggles] : []),
			workbenchSettings.toggleAction,
			...(workbench.canUsePopouts
				? [workbenchSettings.popouts, workbenchSettings.popoutRestore]
				: []),
		],
	}
}

/** Reads and writes the workbench's settings on the workbench itself, which saves them with its layout. */
export function workbenchSettingsStore(workbench: WorkbenchContext): SettingsStore {
	return {
		get<T>(definition: SettingDefinitionBase<T>): T {
			if (definition.id === workbenchSettings.variant.id) return workbench.variant.value as T
			if (definition.id === workbenchSettings.popouts.id) return workbench.popoutsEnabled.value as T
			if (definition.id === workbenchSettings.loneTabs.id) {
				return workbench.hideLoneTabBars.value as T
			}
			if (definition.id === workbenchSettings.toggleAction.id) {
				return workbench.toggleAction.value as T
			}
			if (definition.id === workbenchSettings.popoutRestore.id) {
				return workbench.popoutRestore.value as T
			}
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
			} else if (definition.id === workbenchSettings.loneTabs.id) {
				workbench.setHideLoneTabBars(value as boolean)
			} else if (definition.id === workbenchSettings.toggleAction.id) {
				workbench.setToggleAction(value as WorkbenchToggleAction)
			} else if (definition.id === workbenchSettings.popoutRestore.id) {
				workbench.setPopoutRestore(value as WorkbenchPopoutRestore)
			} else if (definition.id === workbenchSettings.toggles.id) {
				workbench.setToggleLocation(value ? 'tabs' : 'menu')
			}
		},
	}
}
