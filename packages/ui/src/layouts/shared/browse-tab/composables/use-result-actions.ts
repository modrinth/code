import { CheckIcon, PlusIcon, SpinnerIcon } from '@modrinth/assets'

import { useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import type { BrowseManagerContext } from '../providers/browse-manager'
import type { BrowseHit, BrowseInstallState, CardAction } from '../types'

export function useResultActions(ctx: BrowseManagerContext) {
	const { formatMessage } = useVIntl()

	function stateLabel(hit: BrowseHit, state: BrowseInstallState) {
		switch (state) {
			case 'installed':
				return formatMessage(commonMessages.installedLabel)
			case 'selected':
			case 'selected-dependency':
				return formatMessage(commonMessages.selectedLabel)
			case 'validating':
				return formatMessage(commonMessages.validatingLabel)
			case 'installing':
				return formatMessage(commonMessages.installingLabel)
			case 'idle':
				return ctx.install?.idleLabel?.(hit) ?? formatMessage(commonMessages.installButton)
		}
	}

	function installAction(hit: BrowseHit, state: BrowseInstallState): CardAction {
		const busy = state === 'installing' || state === 'validating'
		const selected = state === 'selected' || state === 'selected-dependency'

		return {
			key: 'install',
			label: stateLabel(hit, state),
			icon: busy
				? SpinnerIcon
				: selected || state === 'installed'
					? CheckIcon
					: (ctx.install?.idleIcon ?? PlusIcon),
			iconClass: busy ? 'animate-spin' : undefined,
			disabled:
				busy ||
				state === 'installed' ||
				state === 'selected-dependency' ||
				(ctx.selection?.installing.value ?? false),
			color: selected ? 'green' : 'brand',
			type: 'outlined',
			onClick: () => ctx.install?.run(hit),
		}
	}

	return function actionsFor(hit: BrowseHit): CardAction[] {
		const state = ctx.install?.state(hit)
		return [...(state ? [installAction(hit, state)] : []), ...(ctx.actions?.(hit) ?? [])]
	}
}
