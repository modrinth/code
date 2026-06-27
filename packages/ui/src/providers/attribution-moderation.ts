import { createContext } from './create-context'

export interface QuickReply<T = undefined> {
	label: string
	message: string | ((context: T) => Promise<string> | string)
	shouldShow?: (context: T) => boolean
	private?: boolean
}

export interface AttributionModerationContext {
	attributionQuickReplies: ReadonlyArray<QuickReply>
}

export const [injectAttributionModeration, provideAttributionModeration] =
	createContext<AttributionModerationContext>('AttributionModeration')
