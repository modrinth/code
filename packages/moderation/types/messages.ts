export interface WeightedMessage {
  /**
   * The weight of the action's active message, used to determine the place where the message is placed in the final moderation message.
   */
  weight: number

  /**
   * The message which is appended to the final moderation message if the button is active.
   * @returns A function that lazily loads the message which is appended if the button is active.
   * @example () => import('./messages/blah/something.content.ts).then(m => m.html)
   */
  message: () => Promise<typeof import('*.md?raw') | string>
}
