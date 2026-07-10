import type { KeybindListener } from '../types/keybinds'

const keybinds: { [id: string]: KeybindListener } = {
	'next-stage': {
		keybind: 'ArrowRight',
		description: 'Go to next stage',
		enabled: (ctx) => !ctx.state.isDone && !ctx.state.hasGeneratedMessage,
		action: (ctx) => ctx.actions.tryGoNext(),
	},
	'previous-stage': {
		keybind: 'ArrowLeft',
		description: 'Go to previous stage',
		enabled: (ctx) => !ctx.state.isDone && !ctx.state.hasGeneratedMessage,
		action: (ctx) => ctx.actions.tryGoBack(),
	},
	'generate-message': {
		keybind: 'Ctrl+Shift+E',
		description: 'Generate moderation message',
		action: (ctx) => ctx.actions.tryGenerateMessage(),
	},
	'toggle-collapse': {
		keybind: 'Shift+C',
		description: 'Toggle collapse/expand',
		action: (ctx) => ctx.actions.tryToggleCollapse(),
	},
	'reset-progress': {
		keybind: 'Ctrl+Shift+R',
		description: 'Reset moderation progress',
		action: (ctx) => ctx.actions.tryResetProgress(),
	},
	'skip-project': {
		keybind: 'Ctrl+Shift+S',
		description: 'Skip to next project',
		enabled: (ctx) => ctx.state.futureProjectCount > 0 && !ctx.state.isDone,
		action: (ctx) => ctx.actions.trySkipProject(),
	}
}

export default keybinds
