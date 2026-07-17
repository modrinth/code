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
	},
	'copy-permalink': {
		keybind: 'Ctrl+Alt+C',
		description: 'Copy permalink',
		action: (ctx) => ctx.actions.tryCopyLink(true, false, false),
	},
	'copy-relative-permalink': {
		keybind: 'Ctrl+Alt+R',
		description: 'Copy relative permalink',
		action: (ctx) => ctx.actions.tryCopyLink(true, true, false),
	},
	'copy-page-permalink': {
		keybind: 'Shift+Ctrl+Alt+C',
		description: 'Copy permalink with page',
		action: (ctx) => ctx.actions.tryCopyLink(true, false, true),
	},
	'copy-page-relative-permalink': {
		keybind: 'Shift+Ctrl+Alt+R',
		description: 'Copy relative permalink with page',
		action: (ctx) => ctx.actions.tryCopyLink(true, true, true),
	},
	'copy-id': {
		keybind: 'Ctrl+Alt+D',
		description: 'Copy Project ID',
		action: (ctx) => ctx.actions.tryCopyId(),
	},
	'approve-project': {
		keybind: 'Shift+Alt+A',
		description: 'Approve project',
		action: (ctx) => ctx.actions.tryApprove(),
	},
	'withhold-project': {
		keybind: 'Shift+Alt+W',
		description: 'Withhold project',
		action: (ctx) => ctx.actions.tryWithhold(),
	},
	'reject-project': {
		keybind: 'Shift+Alt+R',
		description: 'Reject project',
		action: (ctx) => ctx.actions.tryReject(),
	},
}

export default keybinds
