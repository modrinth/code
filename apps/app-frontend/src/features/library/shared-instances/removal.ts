import { defineMessages } from '@modrinth/ui'

import type { UninstalledSharedInstance } from './types'

export type SharedInstanceRemovalMode = 'unpublish' | 'leave' | 'unpublish_and_leave'

export const getSharedInstanceRemovalMode = (
	sharedInstances: UninstalledSharedInstance[],
): SharedInstanceRemovalMode => {
	const ownsAny = sharedInstances.some((instance) => instance.role === 'owner')
	const ownsAll = sharedInstances.every((instance) => instance.role === 'owner')
	if (ownsAll) return 'unpublish'
	return ownsAny ? 'unpublish_and_leave' : 'leave'
}

export const sharedInstanceRemovalButtonMessages = defineMessages({
	unpublish: { id: 'app.library.instance.action.unpublish', defaultMessage: 'Unpublish' },
	leave: { id: 'app.library.instance.action.leave', defaultMessage: 'Leave' },
	unpublish_and_leave: {
		id: 'app.library.instance.action.unpublish-and-leave',
		defaultMessage: 'Unpublish and leave',
	},
})
