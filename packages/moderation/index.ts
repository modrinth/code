import type { Stage } from './types/stage'

import example from './data/example'

export * from './types/actions'
export * from './types/messages'
export * from './types/stage'

export * from './utils/action-utils'

export const STAGE_INFORMATION: Record<string, ReadonlyArray<Stage>> = {
  example_project_type: example,
}
