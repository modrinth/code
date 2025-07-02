import type { WeightedMessage } from './messages'

export type ActionType = 'button' | 'dropdown' | 'multi-select-chips'
export type Action = ButtonAction | DropdownAction | MultiSelectChipsAction

export interface BaseAction {
  /**
   * The type of action, which determines how the action is presented to the moderator and what it does.
   */
  type: ActionType

  /**
   * Any additional text data that is required to complete the action.
   */
  relevantExtraInput?: AdditionalTextInput[]
}

/**
 * Represents a button action, which is a simple toggle button that can be used to append a message to the final moderation message.
 */
export interface ButtonAction extends BaseAction, WeightedMessage {
  type: 'button'

  /**
   * The label of the button, which is displayed to the moderator. The text on the button.
   */
  label: string
}

export interface DropdownActionOption extends WeightedMessage {
  /**
   * The label of the option, which is displayed to the moderator.
   */
  label: string
}

export interface DropdownAction extends BaseAction {
  type: 'dropdown'

  /**
   * The label associated with the dropdown.
   */
  label: string

  /**
   * The options available in the dropdown.
   */
  options: DropdownActionOption[]

  /**
   * The default option selected in the dropdown, by index.
   */
  defaultOption?: number
}

export interface MultiSelectChipsOption extends WeightedMessage {
  /**
   * The label of the chip, which is displayed to the moderator.
   */
  label: string
}

export interface MultiSelectChipsAction extends BaseAction {
  type: 'multi-select-chips'

  /**
   * The label associated with the multi-select chips.
   */
  label: string

  /**
   * The options available in the multi-select chips.
   */
  options: MultiSelectChipsOption[]
}

export interface AdditionalTextInput {
  /**
   * The label of the text input, which is displayed to the moderator.
   */
  label: string

  /**
   * The placeholder text for the text input.
   */
  placeholder?: string

  /**
   * Whether the text input is required to be filled out before the action can be completed.
   */
  required?: boolean
}
