import { DownloadIcon, LogOutIcon, UnlinkIcon } from '@modrinth/assets'
import { type ButtonMenuOption, useVIntl } from '@modrinth/ui'

import { useHiddenInstances } from '@/features/library/hidden-instances/use-hidden-instances'
import { libraryInstanceMessages } from '@/features/library/messages'

import { injectSharedInstanceActions, type SharedInstanceInstallOptions } from './actions-context'
import { getSharedInstanceRemovalMode, sharedInstanceRemovalButtonMessages } from './removal'
import type { UninstalledSharedInstance } from './types'

export function useSharedInstanceMenu() {
	const { formatMessage } = useVIntl()
	const { install, requestRemove } = injectSharedInstanceActions()
	const { getVisibilityOption } = useHiddenInstances()

	const getSharedInstanceOptions = (
		instance: UninstalledSharedInstance,
		installOptions?: SharedInstanceInstallOptions,
	): ButtonMenuOption[] => [
		{
			id: 'install',
			label: formatMessage(libraryInstanceMessages.install),
			icon: DownloadIcon,
			tone: 'brand',
			action: () => install(instance, installOptions),
		},
		getVisibilityOption(instance.id),
		{ type: 'divider' },
		{
			id: instance.role === 'owner' ? 'unpublish' : 'leave',
			label: formatMessage(
				sharedInstanceRemovalButtonMessages[getSharedInstanceRemovalMode([instance])],
			),
			icon: instance.role === 'owner' ? UnlinkIcon : LogOutIcon,
			tone: 'orange',
			hoverFilledOnly: true,
			action: () => requestRemove([instance]),
		},
	]

	return { getSharedInstanceOptions }
}
