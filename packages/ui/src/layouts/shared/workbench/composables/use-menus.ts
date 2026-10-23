import {
	ArrowUpDownIcon,
	ExternalIcon,
	EyeOffIcon,
	FoldVerticalIcon,
	LayersIcon,
	LayoutTemplateIcon,
	LinkIcon,
	MoveIcon,
	PanelTopCloseIcon,
	SplitIcon,
	UndoIcon,
	UnfoldHorizontalIcon,
	UnfoldVerticalIcon,
	UnlinkIcon,
	XIcon,
} from '@modrinth/assets'
import { computed } from 'vue'

import type { ButtonMenuLeafOption, ButtonMenuOption } from '#ui/components/base/buttons/types'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import type { WorkbenchContext } from '../providers/workbench'
import type { WorkbenchHost, WorkbenchPanel, WorkbenchSplitSide } from '../types'

const messages = defineMessages({
	moveTo: { id: 'workbench.menu.move-to', defaultMessage: 'Move to' },
	center: { id: 'workbench.menu.region.center', defaultMessage: 'Center' },
	left: { id: 'workbench.menu.region.left', defaultMessage: 'Left panel' },
	right: { id: 'workbench.menu.region.right', defaultMessage: 'Right panel' },
	bottom: { id: 'workbench.menu.region.bottom', defaultMessage: 'Bottom panel' },
	split: { id: 'workbench.menu.split', defaultMessage: 'Split' },
	splitRight: { id: 'workbench.menu.split.right', defaultMessage: 'To the right' },
	splitLeft: { id: 'workbench.menu.split.left', defaultMessage: 'To the left' },
	splitBelow: { id: 'workbench.menu.split.below', defaultMessage: 'Below' },
	splitAbove: { id: 'workbench.menu.split.above', defaultMessage: 'Above' },
	popout: { id: 'workbench.menu.popout', defaultMessage: 'Open in its own window' },
	popoutSelected: {
		id: 'workbench.menu.popout-selected',
		defaultMessage: 'Open {count} tabs in their own window',
	},
	dock: { id: 'workbench.menu.dock', defaultMessage: 'Return to the main window' },
	openPanel: { id: 'workbench.menu.open-panel', defaultMessage: 'Open panel' },
	closePanel: { id: 'workbench.menu.close-panel', defaultMessage: 'Close panel' },
	closeSelected: { id: 'workbench.menu.close-selected', defaultMessage: 'Close {count} tabs' },
	selected: { id: 'workbench.menu.selected', defaultMessage: '{count} tabs selected' },
	area: { id: 'workbench.menu.area', defaultMessage: 'Area' },
	hideRegion: { id: 'workbench.menu.hide-region', defaultMessage: 'Hide' },
	collapseRegion: { id: 'workbench.menu.collapse-region', defaultMessage: 'Collapse to tabs' },
	expandRegion: { id: 'workbench.menu.expand-region', defaultMessage: 'Expand' },
	hideBar: { id: 'workbench.menu.hide-bar', defaultMessage: 'Hide tab bar' },
	reverseGroups: { id: 'workbench.menu.reverse-groups', defaultMessage: 'Reverse group order' },
	panels: { id: 'workbench.menu.panels', defaultMessage: 'Panels' },
	togglePanel: { id: 'workbench.menu.toggle-panel', defaultMessage: 'Toggle panel' },
	linkWith: { id: 'workbench.menu.link-with', defaultMessage: 'Link with' },
	linkSelected: { id: 'workbench.menu.link-selected', defaultMessage: 'Link {count} tabs' },
	group: { id: 'workbench.menu.group', defaultMessage: 'Group' },
	separateAll: { id: 'workbench.menu.separate-all', defaultMessage: 'Separate all' },
	takeOut: { id: 'workbench.menu.take-out', defaultMessage: 'Take out {name}' },
	sideBySide: { id: 'workbench.menu.side-by-side', defaultMessage: 'Side by side' },
	stacked: { id: 'workbench.menu.stacked', defaultMessage: 'One above the other' },
	reverseLink: { id: 'workbench.menu.reverse-link', defaultMessage: 'Reverse linked order' },
})

const hostMessages = {
	center: messages.center,
	left: messages.left,
	right: messages.right,
	bottom: messages.bottom,
} satisfies Record<WorkbenchHost, unknown>

const splitMessages = {
	right: messages.splitRight,
	left: messages.splitLeft,
	bottom: messages.splitBelow,
	top: messages.splitAbove,
} satisfies Record<WorkbenchSplitSide, unknown>

/**
 * The workbench's menus, opened with `open` (e.g. a context menu's), offering what the mode
 * allows. Closed panels can be opened from each of them, which is how they come back. A tab's
 * menu acts on every selected tab when it's one of them.
 */
export function useWorkbenchMenus(
	workbench: WorkbenchContext,
	open: (event: MouseEvent, options: ButtonMenuOption[]) => void,
): Pick<
	WorkbenchContext,
	'openPanelMenu' | 'openRegionMenu' | 'panelsMenuOptions' | 'documentMoveOption'
> {
	const { formatMessage } = useVIntl()
	const { mode } = workbench

	function hosts(): WorkbenchHost[] {
		return ['center', ...workbench.usedRegions.value]
	}

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
			options: hosts()
				.filter((host) => host !== current)
				.map((host) => ({
					id: `move-${host}`,
					label: formatMessage(hostMessages[host]),
					action: () => move(host),
				})),
		} satisfies ButtonMenuOption
	}

	/** Splits along a dock's length first: below or above in the narrow side regions, else beside. */
	function splitOption(host: WorkbenchHost, ids: string[]): ButtonMenuOption {
		const sides: WorkbenchSplitSide[] =
			host === 'left' || host === 'right'
				? ['bottom', 'top', 'right', 'left']
				: ['right', 'left', 'bottom', 'top']
		return {
			type: 'submenu',
			id: 'split',
			label: formatMessage(messages.split),
			icon: SplitIcon,
			disabled: !workbench.canSplit(ids),
			options: sides.map((side) => ({
				id: `split-${side}`,
				label: formatMessage(splitMessages[side]),
				action: () => workbench.splitPanels(ids, side),
			})),
		}
	}

	/**
	 * What can be done with the area a tab bar is in: collapsing, expanding or hiding a side
	 * region (the center is always shown), hiding the tab bar's group's bar, and turning the
	 * area's groups around. A popout window only has its tab bar to hide.
	 */
	function areaOption(
		host: WorkbenchHost,
		panelId: string | undefined,
		inPopout: boolean,
	): ButtonMenuOption | null {
		const options: ButtonMenuLeafOption[] = []
		if (host !== 'center' && !inPopout) {
			const collapsed = workbench.collapsed[host].value
			options.push(
				{
					id: collapsed ? 'expand-region' : 'collapse-region',
					label: formatMessage(collapsed ? messages.expandRegion : messages.collapseRegion),
					icon: FoldVerticalIcon,
					action: () => workbench.setRegionCollapsed(host, !collapsed),
				},
				{
					id: 'hide-region',
					label: formatMessage(messages.hideRegion),
					icon: EyeOffIcon,
					action: () => workbench.setRegionVisible(host, false),
				},
			)
		}
		if (panelId && workbench.canHideTabBar(host)) {
			options.push({
				id: 'hide-bar',
				label: formatMessage(messages.hideBar),
				icon: PanelTopCloseIcon,
				action: () => workbench.hideTabBar(panelId),
			})
		}
		if (!inPopout && workbench.canReverseGroups(host)) {
			options.push({
				id: 'reverse-groups',
				label: formatMessage(messages.reverseGroups),
				icon: ArrowUpDownIcon,
				action: () => workbench.reverseGroups(host),
			})
		}
		if (!options.length) return null
		return {
			type: 'submenu',
			id: 'area',
			label: formatMessage(messages.area),
			icon: LayoutTemplateIcon,
			options,
		}
	}

	/** What can be done with a linked tab's panels: how they're laid out, and separating them. */
	function linkGroupOption(tab: WorkbenchPanel): ButtonMenuOption {
		const link = workbench.links.value.find((candidate) => candidate.id === tab.id)
		const sideBySide = link?.orientation === 'horizontal'
		return {
			type: 'submenu',
			id: 'group',
			label: formatMessage(messages.group),
			icon: LinkIcon,
			options: [
				{
					id: 'link-orientation',
					label: formatMessage(sideBySide ? messages.stacked : messages.sideBySide),
					icon: sideBySide ? UnfoldVerticalIcon : UnfoldHorizontalIcon,
					action: () =>
						workbench.setLinkOrientation(tab.id, sideBySide ? 'vertical' : 'horizontal'),
				},
				{
					id: 'reverse-link',
					label: formatMessage(messages.reverseLink),
					icon: ArrowUpDownIcon,
					action: () => workbench.reverseLink(tab.id),
				},
				{ type: 'divider' },
				{
					id: 'separate-all',
					label: formatMessage(messages.separateAll),
					icon: UnlinkIcon,
					action: () => workbench.unlinkAll(tab.id),
				},
				...(tab.members ?? []).map((member) => ({
					id: `take-out-${member.id}`,
					label: formatMessage(messages.takeOut, { name: member.title }),
					icon: member.icon,
					action: () => workbench.unlinkPanel(member.id),
				})),
			],
		}
	}

	function openPanelMenu(event: MouseEvent, id: string) {
		const panel = workbench.panels.value.find((candidate) => candidate.id === id)
		if (!panel) return
		const host = workbench.regionOf(panel)
		const detached = workbench.isDetached(id)
		const selection = workbench.selectedPanels.value
		const ids = selection.length > 1 && selection.includes(id) ? [...selection] : [id]
		const several = ids.length > 1
		const closedOptions = mode.closePanels ? openClosedOptions(host) : []
		const options: ButtonMenuOption[] = []

		if (several) {
			options.push({
				type: 'heading',
				id: 'selected',
				label: formatMessage(messages.selected, { count: ids.length }),
			})
		}
		if (mode.movePanels && !detached) {
			options.push(
				moveToOption(host, (target) => {
					for (const each of ids) workbench.movePanel(each, target)
				}),
			)
			if (several) {
				options.push({
					id: 'link-selected',
					label: formatMessage(messages.linkSelected, { count: ids.length }),
					icon: LinkIcon,
					action: () => {
						for (const other of ids.slice(1)) workbench.linkPanels(ids[0], other)
					},
				})
			} else {
				const others = workbench.panels.value.filter(
					(other) => other.id !== id && workbench.isOpen(other.id),
				)
				if (others.length) {
					options.push({
						type: 'submenu',
						id: 'link-with',
						label: formatMessage(messages.linkWith),
						icon: LinkIcon,
						options: others.map((other) => ({
							id: `link-${other.id}`,
							label: other.title,
							icon: other.icon,
							action: () => workbench.linkPanels(id, other.id),
						})),
					})
				}
			}
		}
		if (panel.members && !several) options.push(linkGroupOption(panel))
		if (mode.splitPanels && !mode.headerless.includes(host) && !detached) {
			options.push(splitOption(host, ids))
		}
		if (detached) {
			options.push({
				id: 'dock',
				label: formatMessage(messages.dock),
				icon: UndoIcon,
				action: () => {
					for (const each of ids) workbench.dockPanel(each)
				},
			})
		} else if (workbench.canPopout(id)) {
			options.push({
				id: 'popout',
				label: several
					? formatMessage(messages.popoutSelected, { count: ids.length })
					: formatMessage(messages.popout),
				icon: ExternalIcon,
				action: () => void workbench.popoutPanels(ids),
			})
		}
		const area = areaOption(host, id, detached)
		if (area) options.push({ type: 'divider' }, area)
		if (closedOptions.length) {
			options.push(
				{ type: 'divider' },
				{
					type: 'submenu',
					id: 'open-panel',
					label: formatMessage(messages.openPanel),
					icon: LayersIcon,
					options: closedOptions,
				},
			)
		}
		if (mode.closePanels) {
			options.push(
				{ type: 'divider' },
				{
					id: 'close-panel',
					label: several
						? formatMessage(messages.closeSelected, { count: ids.length })
						: formatMessage(messages.closePanel),
					icon: XIcon,
					action: () => {
						for (const each of ids) workbench.closePanel(each)
					},
				},
			)
		}
		if (options.some((option) => option.type !== 'divider')) open(event, options)
	}

	/** The menu of a tab bar's empty part; `panelId` is a panel of its group, if it holds any. */
	function openRegionMenu(event: MouseEvent, host: WorkbenchHost, panelId?: string) {
		const closedOptions = mode.closePanels ? openClosedOptions(host) : []
		const options: ButtonMenuOption[] = []
		if (closedOptions.length) {
			options.push({
				type: 'submenu',
				id: 'open-panel',
				label: formatMessage(messages.openPanel),
				icon: LayersIcon,
				options: closedOptions,
			})
		}
		const area = areaOption(host, panelId, !!event.view && event.view !== window)
		if (area) options.push(area)
		if (options.length) open(event, options)
	}

	/** Every panel, to open or close each, in a submenu, e.g. for a toolbar menu. */
	const panelsMenuOptions = computed<ButtonMenuOption[]>(() => [
		{
			type: 'submenu',
			id: 'toggle-panel',
			label: formatMessage(messages.togglePanel),
			icon: LayersIcon,
			options: workbench.panels.value.map((panel) => {
				const isOpen = workbench.isOpen(panel.id)
				return {
					id: `panel-${panel.id}`,
					label: panel.title,
					icon: panel.icon,
					selected: isOpen,
					action: () =>
						isOpen && mode.closePanels
							? workbench.closePanel(panel.id)
							: workbench.revealPanel(panel.id),
				}
			}),
		},
	])

	function documentMoveOption(id: string) {
		const host = workbench.hostOfDocument(id)
		if (!host || !mode.movePanels) return null
		return moveToOption(host, (target) => workbench.moveDocument(id, target))
	}

	return { openPanelMenu, openRegionMenu, panelsMenuOptions, documentMoveOption }
}
