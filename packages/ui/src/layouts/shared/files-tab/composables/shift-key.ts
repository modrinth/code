import { createSharedComposable, useKeyModifier } from '@vueuse/core'

/** Whether Shift is held, shared so every file row reuses one set of listeners. */
export const useShiftKey = createSharedComposable(() => useKeyModifier('Shift', { initial: false }))
