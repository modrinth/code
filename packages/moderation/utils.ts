import type {
  Action,
  AdditionalTextInput,
  ButtonAction,
  ConditionalMessage,
  ToggleAction,
} from './types/actions'

export interface ActionState {
  selected: boolean
  value?: Set<number> | number | string | unknown
}

export interface MessagePart {
  weight: number
  content: string
  actionId: string
  stageIndex: number
}

export type SerializedActionState = {
  isSet?: boolean
} & ActionState

export function getActionIdForStage(
  action: Action,
  stageIndex: number,
  actionIndex?: number,
  enabledIndex?: number,
): string {
  if (action.id) {
    return `stage-${stageIndex}-${action.id}`
  }
  const suffix = enabledIndex !== undefined ? `-enabled-${enabledIndex}` : ''
  return `stage-${stageIndex}-action-${actionIndex}${suffix}`
}

export function getActionId(action: Action, currentStage: number, index?: number): string {
  return getActionIdForStage(action, currentStage, index)
}

export function getActionKey(
  action: Action,
  currentStage: number,
  visibleActions: Action[],
): string {
  const index = visibleActions.indexOf(action)
  return `${currentStage}-${index}-${getActionId(action, currentStage)}`
}

export function serializeActionStates(states: Record<string, ActionState>): string {
  const serializable: Record<string, SerializedActionState> = {}
  for (const [key, state] of Object.entries(states)) {
    serializable[key] = {
      selected: state.selected,
      value: state.value instanceof Set ? Array.from(state.value) : state.value,
      isSet: state.value instanceof Set,
    }
  }
  return JSON.stringify(serializable)
}

export function deserializeActionStates(data: string): Record<string, ActionState> {
  try {
    const parsed = JSON.parse(data)
    const states: Record<string, ActionState> = {}
    for (const [key, state] of Object.entries(parsed as Record<string, SerializedActionState>)) {
      states[key] = {
        selected: state.selected,
        value: state.isSet ? new Set(state.value as unknown[]) : state.value,
      }
    }
    return states
  } catch {
    return {}
  }
}

export function initializeActionState(action: Action): ActionState {
  if (action.type === 'toggle') {
    return {
      selected: action.defaultChecked || false,
    }
  } else if (action.type === 'dropdown') {
    return {
      selected: true,
      value: action.defaultOption || 0,
    }
  } else if (action.type === 'multi-select-chips') {
    return {
      selected: false,
      value: new Set<number>(),
    }
  } else {
    return {
      selected: false,
    }
  }
}

export function processMessage(
  message: string,
  action: Action,
  stageIndex: number,
  textInputValues: Record<string, string>,
): string {
  let processedMessage = message

  if (action.relevantExtraInput) {
    action.relevantExtraInput.forEach((input, index) => {
      if (input.variable) {
        const inputKey = `stage-${stageIndex}-${action.id || `action-${index}`}-${index}`
        const value = textInputValues[inputKey] || ''

        const regex = new RegExp(`%${input.variable}%`, 'g')
        processedMessage = processedMessage.replace(regex, value)
      }
    })
  }

  return processedMessage
}

export function findMatchingVariant(
  variants: ConditionalMessage[],
  selectedActionIds: string[],
): ConditionalMessage | null {
  for (const variant of variants) {
    const conditions = variant.conditions

    const meetsRequired =
      !conditions.requiredActions ||
      conditions.requiredActions.every((id) => selectedActionIds.includes(id))

    const meetsExcluded =
      !conditions.excludedActions ||
      !conditions.excludedActions.some((id) => selectedActionIds.includes(id))

    if (meetsRequired && meetsExcluded) {
      return variant
    }
  }

  return variants.find((v) => v.fallbackMessage) || null
}

export async function getActionMessage(
  action: ButtonAction | ToggleAction,
  selectedActionIds: string[],
): Promise<string> {
  if (action.conditionalMessages && action.conditionalMessages.length > 0) {
    const matchingConditional = findMatchingVariant(action.conditionalMessages, selectedActionIds)
    if (matchingConditional) {
      return (await matchingConditional.message()) as string
    }
  }

  return (await action.message()) as string
}

export function getVisibleInputs(
  action: Action,
  actionStates: Record<string, ActionState>,
): AdditionalTextInput[] {
  if (!action.relevantExtraInput) return []

  const selectedActionIds = Object.entries(actionStates)
    .filter(([, state]) => state.selected)
    .map(([id]) => id)

  return action.relevantExtraInput.filter((input) => {
    if (!input.showWhen) return true

    const meetsRequired =
      !input.showWhen.requiredActions ||
      input.showWhen.requiredActions.every((id) => selectedActionIds.includes(id))

    const meetsExcluded =
      !input.showWhen.excludedActions ||
      !input.showWhen.excludedActions.some((id) => selectedActionIds.includes(id))

    return meetsRequired && meetsExcluded
  })
}
