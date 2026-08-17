import { createContext } from '@modrinth/ui'

import type { AppEvent } from '@/generated/app-events/AppEvent'

export type AppEventType = AppEvent['type']
export type AppEventPayload<Type extends AppEventType> = Extract<
	AppEvent,
	{ type: Type }
>['payload']
export type AppEventHandler<Type extends AppEventType> = (
	payload: AppEventPayload<Type>,
) => void | Promise<void>

export interface AppEvents {
	on<Type extends AppEventType>(type: Type, handler: AppEventHandler<Type>): () => void
	once<Type extends AppEventType>(type: Type, handler: AppEventHandler<Type>): () => void
}

export const [injectAppEvents, provideAppEvents] = createContext<AppEvents>('root', 'appEvents')
