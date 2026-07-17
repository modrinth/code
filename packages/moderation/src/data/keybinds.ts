import type { KeybindListener } from '../types/keybinds'
import type { Labrinth } from '@modrinth/api-client'

const copyProjectLink = async (
	project: Labrinth.Projects.v2.Project,
	permalink: boolean,
	relative: boolean,
	page: boolean
) => {
	let url = ``
	if (relative) {
		url += `${globalThis.location.origin}`
	} else {
		url += `https://modrinth.com`
	}

	if (permalink) {
		url += `/project/${project.id}`
	} else {
		url += `/${project.project_type}/${project.slug}`
	}

	if (page) {
		url += `/${globalThis.location.pathname.split('/').slice(3).join('/')}`
	}

	await navigator.clipboard.writeText(url)
}

const keybinds: { [id: string]: KeybindListener } = {
	'next-stage': {
		keybind: 'ArrowRight',
		description: 'Go to next stage',
		scope: 'checklist',
		enabled: (ctx) => !ctx.state.isDone,
		action: (ctx) => ctx.actions.tryGoNext(),
	},
	'previous-stage': {
		keybind: 'ArrowLeft',
		description: 'Go to previous stage',
		scope: 'checklist',
		enabled: (ctx) => !ctx.state.isDone,
		action: (ctx) => ctx.actions.tryGoBack(),
	},
	'generate-message': {
		keybind: 'Ctrl+Shift+E',
		description: 'Generate moderation message',
		scope: 'checklist',
		action: (ctx) => ctx.actions.tryGenerateMessage(),
	},
	'toggle-collapse': {
		keybind: 'Shift+C',
		description: 'Toggle collapse/expand',
		scope: 'checklist',
		action: (ctx) => ctx.actions.tryToggleCollapse(),
	},
	'reset-progress': {
		keybind: 'Ctrl+Shift+R',
		description: 'Reset moderation progress',
		scope: 'checklist',
		action: (ctx) => ctx.actions.tryResetProgress(),
	},
	'skip-project': {
		keybind: 'Ctrl+Shift+S',
		description: 'Skip to next project',
		scope: 'checklist',
		enabled: (ctx) => ctx.state.futureProjectCount > 0 && !ctx.state.isDone,
		action: (ctx) => ctx.actions.trySkipProject(),
	},
	'copy-permalink': {
		keybind: 'Ctrl+Alt+C',
		description: 'Copy permalink',
		scope: 'project',
		action: async (ctx) => copyProjectLink(ctx.project, true, false, false),
	},
	'copy-relative-permalink': {
		keybind: 'Ctrl+Alt+R',
		description: 'Copy relative permalink',
		scope: 'project',
		action: async (ctx) => copyProjectLink(ctx.project, true, true, false),
	},
	'copy-page-permalink': {
		keybind: 'Shift+Ctrl+Alt+C',
		description: 'Copy permalink with page',
		scope: 'project',
		action: async (ctx) => copyProjectLink(ctx.project, true, false, true),
	},
	'copy-page-relative-permalink': {
		keybind: 'Shift+Ctrl+Alt+R',
		description: 'Copy relative permalink with page',
		scope: 'project',
		action: async (ctx) => copyProjectLink(ctx.project, true, true, true),
	},
	'copy-id': {
		keybind: 'Ctrl+Alt+D',
		description: 'Copy Project ID',
		scope: 'project',
		action: async (ctx) => await navigator.clipboard.writeText(ctx.project.id),
	},
	'approve-project': {
		keybind: 'Shift+Alt+A',
		description: 'Approve project',
		scope: 'checklist',
		action: (ctx) => ctx.actions.tryApprove(),
	},
	'withhold-project': {
		keybind: 'Shift+Alt+W',
		description: 'Withhold project',
		scope: 'checklist',
		action: (ctx) => ctx.actions.tryWithhold(),
	},
	'reject-project': {
		keybind: 'Shift+Alt+R',
		description: 'Reject project',
		scope: 'checklist',
		action: (ctx) => ctx.actions.tryReject(),
	},
}

export default keybinds
