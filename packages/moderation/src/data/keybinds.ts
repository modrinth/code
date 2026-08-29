import type { Labrinth } from '@modrinth/api-client'

import type { KeybindListener } from '../types/keybinds'

const copyProjectLink = async (
	project: Labrinth.Projects.v2.Project,
	permalink: boolean,
	relative: boolean,
	page: boolean,
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
	return url
}

function isOfficialModrinthHost(): boolean {
	const host = globalThis.location?.hostname
	return host === 'modrinth.com' || host === 'www.modrinth.com' || host === 'staging.modrinth.com'
}

function isLocalhost(): boolean {
	const host = globalThis.location?.hostname
	return host === 'localhost' || host === '127.0.0.1' || host === '[::1]'
}

const keybinds: { [id: string]: KeybindListener } = {
	'next-stage': {
		keybind: 'ArrowRight',
		description: 'Go to next stage',
		scope: 'checklist',
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
		action: async (ctx) => {
			const url = await copyProjectLink(ctx.project, true, false, false)
			ctx.notifyCopied(url, 'Copied permalink to clipboard')
		},
	},
	'copy-relative-permalink': {
		keybind: 'Ctrl+Alt+R',
		description: 'Copy relative permalink',
		scope: 'project',
		action: async (ctx) => {
			const url = await copyProjectLink(ctx.project, true, true, false)
			ctx.notifyCopied(url, 'Copied relative permalink to clipboard')
		},
	},
	'copy-page-permalink': {
		keybind: 'Shift+Ctrl+Alt+C',
		description: 'Copy permalink with page',
		scope: 'project',
		action: async (ctx) => {
			const url = await copyProjectLink(ctx.project, true, false, true)
			ctx.notifyCopied(url, 'Copied permalink with page to clipboard')
		},
	},
	'copy-page-relative-permalink': {
		keybind: 'Shift+Ctrl+Alt+R',
		description: 'Copy relative permalink with page',
		scope: 'project',
		action: async (ctx) => {
			const url = await copyProjectLink(ctx.project, true, true, true)
			ctx.notifyCopied(url, 'Copied relative permalink with page to clipboard')
		},
	},
	'copy-id': {
		keybind: 'Ctrl+Alt+D',
		description: 'Copy Project ID',
		scope: 'project',
		action: async (ctx) => {
			await navigator.clipboard.writeText(ctx.project.id)
			ctx.notifyCopied(ctx.project.id, 'Copied Project ID to clipboard')
		},
	},
	'open-official-site': {
		keybind: 'Ctrl+Shift+P',
		description: 'Open current page on production/staging',
		scope: 'global',
		enabled: () => !isOfficialModrinthHost(),
		action: (ctx) => {
			globalThis.open(ctx.officialUrl, '_blank', 'noopener,noreferrer')
		},
	},
	'open-localhost': {
		keybind: [],
		description: 'Open current page on localhost',
		scope: 'global',
		enabled: () => !isLocalhost(),
		action: (ctx) => {
			globalThis.open(ctx.localhostUrl, '_blank', 'noopener,noreferrer')
		},
	},
	'copy-official-site': {
		keybind: 'Ctrl+Shift+O',
		description: 'Copy production/staging URL (localhost only)',
		scope: 'global',
		enabled: () => isLocalhost(),
		action: async (ctx) => {
			await navigator.clipboard.writeText(ctx.officialUrl)
			const environment = ctx.officialUrl.startsWith('https://staging.modrinth.com')
				? 'staging'
				: 'production'
			ctx.notifyCopied(ctx.officialUrl, `Copied ${environment} URL to clipboard`)
		},
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
	'tech-review-top': {
		keybind: 'ArrowUp',
		description: 'Go to top of the tech review card',
		scope: 'tech-review',
		action: (ctx) => ctx.actions.goToTop(),
	},
	'tech-review-bottom': {
		keybind: 'ArrowDown',
		description: 'Go to bottom of the tech review card',
		scope: 'tech-review',
		action: (ctx) => ctx.actions.goToBottom(),
	},
}

export default keybinds
