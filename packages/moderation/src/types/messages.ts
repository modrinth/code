export interface WeightedMessage {
	/**
	 * The weight of the action's active message, used to determine the place where the message is placed in the final moderation message.
	 */
	weight: number

	/**
	 * The message which is appended to the final moderation message if the button is active.
	 * @returns A function that lazily loads the message which is appended if the button is active.
	 * @example async () => (await import('../messages/example.md?raw')).default,
	 */
	message: () => Promise<string>
}
