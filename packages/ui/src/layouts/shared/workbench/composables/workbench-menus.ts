import { EyeOffIcon, LayersIcon, MoveIcon, SplitIcon, XIcon } from '@modrinth/assets'

import type { ButtonMenuLeafOption, ButtonMenuOption } from '#ui/components/base/buttons/types'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import type { WorkbenchContext, WorkbenchLayoutState } from '../providers/workbench'
import { WORKBENCH_HOSTS, type WorkbenchHost } from '../types'

const messages = defineMessages({
	moveTo: { id: 'workbench.menu.move-to', defaultMessage: 'Move to' },
	center: { id: 'workbench.menu.region.center', defaultMessage: 'Center' },
	left: { id: 'workbench.menu.region.left', defaultMessage: 'Left panel' },
	right: { id: 'workbench.menu.region.right', defaultMessage: 'Right panel' },
	bottom: { id: 'workbench.menu.region.bottom', defaultMessage: 'Bottom panel' },
	splitRight: { id: 'workbench.menu.split-right', defaultMessage: 'Split right' },
	splitDown: { id: 'workbench.menu.split-down', defaultMessage: 'Split down' },
	openPanel: { id: 'workbench.menu.open-panel', defaultMessage: 'Open panel' },
	closePanel: { id: 'workbench.menu.close-panel', defaultMessage: 'Close panel' },
	hideRegion: { id: 'workbench.menu.hide-region', defaultMessage: 'Hide this area' },
	panels: { id: 'workbench.menu.panels', defaultMessage: 'Panels' },
})

const hostMessages = {
	center: messages.center,
	left: messages.left,
	right: messages.right,
	bottom: messages.bottom,
} satisfies Record<WorkbenchHost, unknown>

/**
 * The workbench's menus, opened with `open` (e.g. a context menu's). Closed panels can be opened
 * from each of them, which is how panels are added back once closed.
 */
export function useWorkbenchMenus(
	workbench: WorkbenchLayoutState,
	open: (event: MouseEvent, options: ButtonMenuOption[]) => void,
): Pick<
	WorkbenchContext,
	'openPanelMenu' | 'openRegionMenu' | 'openPanelsMenu' | 'documentMoveOption'
> {
	const { formatMessage } = useVIntl()

	function openClosedOptions(host: WorkbenchHost): ButtonMenuLeafOption[] {
		return workbench.closedPanels.value.map((panel) => ({
			id: `open-${panel.id}`,
			label: panel.title,
			icon: panel.icon,
			action: () => workbench.movePanel(panel.id, host),
		}))
	}

	function moveToOption(current: WorkbenchHost, move: (host: WorkbenchHost) => void) {
		return {
			type: 'submenu',
			id: 'move-to',
			label: formatMessage(messages.moveTo),
			icon: MoveIcon,
			options: WORKBENCH_HOSTS.filter((host) => host !== current).map((host) => ({
				id: `move-${host}`,
				label: formatMessage(hostMessages[host]),
				action: () => move(host),
			})),
		} satisfies ButtonMenuOption
	}

	function openPanelMenu(event: MouseEvent, id: string) {
		const panel = workbench.panels.value.find((candidate) => candidate.id === id)
		if (!panel) return
		const host = workbench.regionOf(panel)
		const closedOptions = openClosedOptions(host)
		const canSplit = workbench.canSplit(id)

		open(event, [
			moveToOption(host, (target) => workbench.movePanel(id, target)),
			{
				id: 'split-right',
				label: formatMessage(messages.splitRight),
				icon: SplitIcon,
				disabled: !canSplit,
				action: () => workbench.splitPanel(id, 'right'),
			},
			{
				id: 'split-down',
				label: formatMessage(messages.splitDown),
				icon: SplitIcon,
				disabled: !canSplit,
				action: () => workbench.splitPanel(id, 'bottom'),
			},
			{ type: 'divider', shown: closedOptions.length > 0 },
			{
				type: 'submenu',
				id: 'open-panel',
				label: formatMessage(messages.openPanel),
				icon: LayersIcon,
				shown: closedOptions.length > 0,
				options: closedOptions,
			},
			{ type: 'divider' },
			{
				id: 'close-panel',
				label: formatMessage(messages.closePanel),
				icon: XIcon,
				action: () => workbench.closePanel(id),
			},
		])
	}

	function openRegionMenu(event: MouseEvent, host: WorkbenchHost) {
		const closedOptions = openClosedOptions(host)
		const options: ButtonMenuOption[] = [
			{
				type: 'submenu',
				id: 'open-panel',
				label: formatMessage(messages.openPanel),
				icon: LayersIcon,
				shown: closedOptions.length > 0,
				options: closedOptions,
			},
		]
		if (host !== 'center') {
			options.push({
				id: 'hide-region',
				label: formatMessage(messages.hideRegion),
				icon: EyeOffIcon,
				action: () => workbench.setRegionVisible(host, false),
			})
		}
		if (closedOptions.length > 0 || host !== 'center') open(event, options)
	}

	function openPanelsMenu(event: MouseEvent) {
		open(event, [
			{ type: 'heading', id: 'panels', label: formatMessage(messages.panels) },
			...workbench.panels.value.map((panel) => {
				const isOpen = workbench.isOpen(panel.id)
				return {
					id: `panel-${panel.id}`,
					label: panel.title,
					icon: panel.icon,
					selected: isOpen,
					action: () =>
						isOpen ? workbench.closePanel(panel.id) : workbench.revealPanel(panel.id),
				}
			}),
		])
	}

	function documentMoveOption(id: string) {
		const host = workbench.hostOfDocument(id)
		if (!host) return null
		return moveToOption(host, (target) => workbench.moveDocument(id, target))
	}

	return { openPanelMenu, openRegionMenu, openPanelsMenu, documentMoveOption }
}
