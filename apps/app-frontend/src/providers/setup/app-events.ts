import { Channel } from '@tauri-apps/api/core'
import { onScopeDispose } from 'vue'

import {
	type AppEventHandler,
	type AppEvents,
	type AppEventType,
	provideAppEvents,
} from '@/providers/app-events'
import { decodeAppEvent } from '@/providers/setup/app-event-codec'

type UntypedAppEventHandler = (payload: unknown) => void | Promise<void>

export function setupAppEventsProvider() {
	const handlers = new Map<AppEventType, Set<UntypedAppEventHandler>>()

	function on<Type extends AppEventType>(type: Type, handler: AppEventHandler<Type>) {
		let eventHandlers = handlers.get(type)
		if (!eventHandlers) {
			eventHandlers = new Set()
			handlers.set(type, eventHandlers)
		}

		const untypedHandler = handler as unknown as UntypedAppEventHandler
		eventHandlers.add(untypedHandler)
		return () => {
			eventHandlers.delete(untypedHandler)
			if (eventHandlers.size === 0) handlers.delete(type)
		}
	}

	function once<Type extends AppEventType>(type: Type, handler: AppEventHandler<Type>) {
		let unsubscribe = () => {}
		unsubscribe = on(type, async (payload) => {
			unsubscribe()
			await handler(payload)
		})
		return unsubscribe
	}

	const events: AppEvents = { on, once }
	const channel = new Channel<ArrayBuffer>((payload) => {
		const event = decodeAppEvent(payload)
		const eventHandlers = handlers.get(event.type)
		if (!eventHandlers) return

		for (const handler of [...eventHandlers]) {
			void Promise.resolve()
				.then(() => handler(event.payload))
				.catch((error) => {
					console.error(`Unhandled ${event.type} app event`, error)
				})
		}
	})

	provideAppEvents(events)

	function dispose() {
		handlers.clear()
	}

	onScopeDispose(dispose)

	return { channel, events, dispose }
}
