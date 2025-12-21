export interface QuickReply<T = undefined> {
	label: string
	message: string | ((context: T) => Promise<string> | string)
	shouldShow?: (context: T) => boolean
	private?: boolean
}
