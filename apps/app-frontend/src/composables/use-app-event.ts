import { onScopeDispose } from 'vue'

import {
	type AppEventHandler,
	type AppEvents,
	type AppEventType,
	injectAppEvents,
} from '@/providers/app-events'

export function useAppEvent<Type extends AppEventType>(
	type: Type,
	handler: AppEventHandler<Type>,
	events: AppEvents = injectAppEvents(),
) {
	const unsubscribe = events.on(type, handler)
	onScopeDispose(unsubscribe)
	return unsubscribe
}
