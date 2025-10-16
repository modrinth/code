import type { Project } from '@modrinth/utils'

import type { WeightedMessage } from './messages'

export type ActionType =
	| 'button'
	| 'dropdown'
	| 'multi-select-chips'
	| 'toggle'
	| 'conditional-button'

export type Action =
	| ButtonAction
	| DropdownAction
	| MultiSelectChipsAction
	| ToggleAction
	| ConditionalButtonAction

export type ModerationStatus = 'approved' | 'rejected' | 'flagged'
export type ModerationSeverity = 'low' | 'medium' | 'high' | 'critical'

export interface BaseAction {
	/**
	 * The type of action, which determines how the action is presented to the moderator and what it does.
	 */
	type: ActionType

	/**
	 * Any additional text data that is required to complete the action.
	 */
	relevantExtraInput?: AdditionalTextInput[]

	/**
	 * Suggested moderation status when this action is selected.
	 */
	suggestedStatus?: ModerationStatus

	/**
	 * Suggested severity level for this moderation action.
	 */
	severity?: ModerationSeverity

	/**
	 * Actions that become available when this action is selected.
	 */
	enablesActions?: Action[]

	/**
	 * Actions that become unavailable when this action is selected.
	 */
	disablesActions?: string[] // Array of action IDs

	/**
	 * Unique identifier for this action, used for conditional logic.
	 */
	id?: string

	/**
	 * A function that determines whether this action should be shown for a given project.
	 *
	 * By default, it returns `true`, meaning the action is always shown.
	 */
	shouldShow?: (project: Project) => boolean
}

/**
 * Represents a conditional message that changes based on other selected actions.
 */
export interface ConditionalMessage extends WeightedMessage {
	/**
	 * Conditions that must be met for this message to be used.
	 */
	conditions: {
		/**
		 * Action IDs that must be selected for this message to apply.
		 */
		requiredActions?: string[]

		/**
		 * Action IDs that must NOT be selected for this message to apply.
		 */
		excludedActions?: string[]
	}
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

	/**
	 * Alternative messages based on other selected actions.
	 */
	conditionalMessages?: ConditionalMessage[]
}

/**
 * Represents a simple toggle/checkbox action with separate layout handling.
 */
export interface ToggleAction extends BaseAction, WeightedMessage {
	type: 'toggle'

	/**
	 * The label of the toggle, which is displayed to the moderator.
	 */
	label: string

	/**
	 * Description text that appears below the toggle.
	 */
	description?: string

	/**
	 * Whether the toggle is checked by default.
	 */
	defaultChecked?: boolean

	/**
	 * Alternative messages based on other selected actions.
	 */
	conditionalMessages?: ConditionalMessage[]
}

/**
 * Represents a button that has different behavior based on other selected actions.
 */
export interface ConditionalButtonAction extends BaseAction {
	type: 'conditional-button'

	/**
	 * The label of the button, which is displayed to the moderator.
	 */
	label: string

	/**
	 * Different message configurations based on conditions.
	 */
	messageVariants: ConditionalMessage[]

	/**
	 * Global fallback message if no variants match their conditions.
	 */
	fallbackMessage?: () => Promise<string>

	/**
	 * The weight of the action's fallback message, used to determine the place where the message is placed in the final moderation message.
	 */
	fallbackWeight?: number
}

export interface DropdownActionOption extends WeightedMessage {
	/**
	 * The label of the option, which is displayed to the moderator.
	 */
	label: string

	/**
	 * A function that determines whether this option should be shown for a given project.
	 *
	 * By default, it returns `true`, meaning the option is always shown.
	 */
	shouldShow?: (project: Project) => boolean
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

	/**
	 * A function that determines whether this option should be shown for a given project.
	 *
	 * By default, it returns `true`, meaning the option is always shown.
	 */
	shouldShow?: (project: Project) => boolean
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

	/**
	 * Whether the text input should use the full markdown editor rather than a simple text input.
	 */
	large?: boolean

	/**
	 * The variable name that will be replaced in the message with the input value.
	 * For example, if variable is "MESSAGE", then "%MESSAGE%" in the action message
	 * will be replaced with the input value.
	 */
	variable?: string

	/**
	 * Conditions that determine when this input is shown.
	 */
	showWhen?: {
		/**
		 * Action IDs that must be selected for this input to be shown.
		 */
		requiredActions?: string[]

		/**
		 * Action IDs that must NOT be selected for this input to be shown.
		 */
		excludedActions?: string[]
	}

	/**
	 * Optional suggestions for the input. Useful for repeating phrases or common responses.
	 */
	suggestions?: string[]
}
