import type { KeybindListener } from '../types/keybinds'

const keybinds: KeybindListener[] = [
  {
    id: 'next-stage',
    keybind: 'ArrowRight',
    description: 'Go to next stage',
    enabled: (ctx) => !ctx.state.isDone && !ctx.state.hasGeneratedMessage,
    action: (ctx) => ctx.actions.tryGoNext(),
  },
  {
    id: 'previous-stage',
    keybind: 'ArrowLeft',
    description: 'Go to previous stage',
    enabled: (ctx) => !ctx.state.isDone && !ctx.state.hasGeneratedMessage,
    action: (ctx) => ctx.actions.tryGoBack(),
  },
  {
    id: 'generate-message',
    keybind: 'Ctrl+Enter',
    description: 'Generate moderation message',
    enabled: (ctx) =>
      !ctx.state.isLoadingMessage && ctx.state.currentStage === ctx.state.totalStages - 1,
    action: (ctx) => ctx.actions.tryGenerateMessage(),
  },
  {
    id: 'toggle-collapse',
    keybind: 'Ctrl+Shift+C',
    description: 'Toggle collapse/expand',
    action: (ctx) => ctx.actions.tryToggleCollapse(),
  },
  {
    id: 'reset-progress',
    keybind: 'Ctrl+Shift+R',
    description: 'Reset moderation progress',
    action: (ctx) => ctx.actions.tryResetProgress(),
  },
  {
    id: 'quick-approve',
    keybind: 'Ctrl+Shift+A',
    description: 'Quick approve',
    enabled: (ctx) => ctx.state.hasGeneratedMessage,
    action: (ctx) => ctx.actions.tryApprove(),
  },
  {
    id: 'quick-reject',
    keybind: 'Ctrl+Shift+X',
    description: 'Quick reject',
    enabled: (ctx) => ctx.state.hasGeneratedMessage,
    action: (ctx) => ctx.actions.tryReject(),
  },
  {
    id: 'skip-project',
    keybind: 'Ctrl+Shift+S',
    description: 'Skip to next project',
    enabled: (ctx) => ctx.state.futureProjectCount > 0 && !ctx.state.isDone,
    action: (ctx) => ctx.actions.trySkipProject(),
  },
]

export default keybinds
