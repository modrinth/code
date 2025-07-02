import type {
  Action,
  ConditionalMessage,
  ModerationSeverity,
  ModerationStatus,
} from '../types/actions'

/**
 * Utility functions for handling moderation actions and their conditional behavior.
 */
export default class ModerationActionUtils {
  /**
   * Evaluates if conditions are met based on selected action IDs.
   */
  static evaluateConditions(
    conditions: ConditionalMessage['conditions'],
    selectedActionIds: string[],
  ): boolean {
    const { requiredActions = [], excludedActions = [] } = conditions

    // Check if all required actions are selected
    const hasRequiredActions = requiredActions.every((id) => selectedActionIds.includes(id))

    // Check if none of the excluded actions are selected
    const hasExcludedActions = excludedActions.some((id) => selectedActionIds.includes(id))

    return hasRequiredActions && !hasExcludedActions
  }

  /**
   * Gets the appropriate message for an action based on selected actions.
   */
  static async getConditionalMessage(action: Action, selectedActionIds: string[]): Promise<string> {
    if ('conditionalMessages' in action && action.conditionalMessages) {
      // Find the first conditional message that matches
      for (const conditionalMsg of action.conditionalMessages) {
        if (this.evaluateConditions(conditionalMsg.conditions, selectedActionIds)) {
          try {
            const result = await conditionalMsg.message()
            return typeof result === 'string' ? result : result.default
          } catch {
            // If conditional message fails, try fallback
            if (conditionalMsg.fallbackMessage) {
              const fallback = await conditionalMsg.fallbackMessage()
              return typeof fallback === 'string' ? fallback : fallback.default
            }
          }
        }
      }
    }

    // Fall back to the default message
    if ('message' in action && action.message) {
      const result = await action.message()
      return typeof result === 'string' ? result : result.default
    }

    return ''
  }

  /**
   * Gets actions that should be enabled based on the current selection.
   */
  static getEnabledActions(selectedActions: Action[]): Action[] {
    const enabledActions: Action[] = []

    selectedActions.forEach((action) => {
      if (action.enablesActions) {
        enabledActions.push(...action.enablesActions)
      }
    })

    return enabledActions
  }

  /**
   * Gets action IDs that should be disabled based on the current selection.
   */
  static getDisabledActionIds(selectedActions: Action[]): string[] {
    const disabledIds: string[] = []

    selectedActions.forEach((action) => {
      if (action.disablesActions) {
        disabledIds.push(...action.disablesActions)
      }
    })

    return disabledIds
  }

  /**
   * Determines the highest suggested severity from selected actions.
   */
  static getSeverity(selectedActions: Action[]): ModerationSeverity | undefined {
    const severityLevels: Record<ModerationSeverity, number> = {
      low: 1,
      medium: 2,
      high: 3,
      critical: 4,
    }

    let highestSeverity: ModerationSeverity | undefined

    selectedActions.forEach((action) => {
      if (action.severity) {
        if (!highestSeverity || severityLevels[action.severity] > severityLevels[highestSeverity]) {
          highestSeverity = action.severity
        }
      }
    })

    return highestSeverity
  }

  /**
   * Determines the suggested status based on selected actions.
   * Priority: rejected > flagged > approved
   */
  static getSuggestedStatus(selectedActions: Action[]): ModerationStatus | undefined {
    const statusPriority: Record<ModerationStatus, number> = {
      approved: 1,
      flagged: 2,
      rejected: 3,
    }

    let highestStatus: ModerationStatus | undefined

    selectedActions.forEach((action) => {
      if (action.suggestedStatus) {
        if (
          !highestStatus ||
          statusPriority[action.suggestedStatus] > statusPriority[highestStatus]
        ) {
          highestStatus = action.suggestedStatus
        }
      }
    })

    return highestStatus
  }

  /**
   * Filters text inputs based on show conditions.
   */
  static getVisibleTextInputs(action: Action, selectedActionIds: string[]) {
    if (!('relevantExtraInput' in action) || !action.relevantExtraInput) {
      return []
    }

    return action.relevantExtraInput.filter((input) => {
      if (!input.showWhen) return true
      return this.evaluateConditions(input.showWhen, selectedActionIds)
    })
  }
}

/**
 * Type guard to check if an action has conditional messages.
 */
export function hasConditionalMessages(
  action: Action,
): action is Action & { conditionalMessages: ConditionalMessage[] } {
  return 'conditionalMessages' in action && Array.isArray(action.conditionalMessages)
}

/**
 * Type guard to check if an action has enablesActions.
 */
export function hasEnablesActions(action: Action): action is Action & { enablesActions: Action[] } {
  return 'enablesActions' in action && Array.isArray(action.enablesActions)
}
