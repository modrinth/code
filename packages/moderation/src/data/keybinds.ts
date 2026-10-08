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

const keybinds: { [id: string]: KeybindListener } = {
	'review-cycle-conversation': {
		keybind: 'Shift+R',
		description: 'Cycle issues, thread, and re-review',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('cycle-conversation'),
		action: (ctx) => ctx.run('cycle-conversation'),
	},
	'review-new-issue': {
		keybind: 'N',
		description: 'Add an issue',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('new-issue'),
		action: (ctx) => ctx.run('new-issue'),
	},
	'review-toggle-left': {
		keybind: 'Shift+ArrowLeft',
		description: 'Toggle project information',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('toggle-left'),
		action: (ctx) => ctx.run('toggle-left'),
	},
	'review-toggle-right': {
		keybind: 'Shift+ArrowRight',
		description: 'Toggle issues and thread panel',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('toggle-right'),
		action: (ctx) => ctx.run('toggle-right'),
	},
	'review-toggle-bottom': {
		keybind: 'Shift+ArrowDown',
		description: 'Toggle tools panel',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('toggle-bottom'),
		action: (ctx) => ctx.run('toggle-bottom'),
	},
	'review-back': {
		keybind: 'Alt+ArrowLeft',
		description: 'Previous project',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('back'),
		action: (ctx) => ctx.run('back'),
	},
	'review-exit': {
		keybind: 'Shift+X',
		description: 'Exit project review',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('exit'),
		action: (ctx) => ctx.run('exit'),
	},
	'review-edit': {
		keybind: 'E',
		description: 'Edit hovered or focused item',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('edit'),
		action: (ctx) => ctx.run('edit'),
	},
	'review-re-review': {
		keybind: 'Alt+R',
		description: 'Open re-review issues',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('re-review'),
		action: (ctx) => ctx.run('re-review'),
	},
	'review-reupload': {
		keybind: 'Alt+U',
		description: 'Open reupload checks',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('reupload'),
		action: (ctx) => ctx.run('reupload'),
	},
	'review-rules': {
		keybind: 'Alt+L',
		description: 'Open rules checks',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('rules'),
		action: (ctx) => ctx.run('rules'),
	},
	'review-post-approval': {
		keybind: 'Alt+S',
		description: 'Open post-approval checks',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('post-approval'),
		action: (ctx) => ctx.run('post-approval'),
	},
	'review-send-reply': {
		keybind: 'Ctrl+Enter',
		description: 'Send reply',
		scope: 'review-composer',
		enabled: (ctx) => ctx.canSend(),
		action: (ctx) => ctx.send('reply'),
	},
	'review-send-note': {
		keybind: 'Ctrl+Shift+Enter',
		description: 'Send private note',
		scope: 'review-composer',
		enabled: (ctx) => ctx.canSend(),
		action: (ctx) => ctx.send('note'),
	},
	'review-tab-description': {
		keybind: 'D',
		description: 'Open description tab',
		scope: 'project-review',
		action: (ctx) => ctx.openTab('description'),
	},
	'review-tab-gallery': {
		keybind: 'G',
		description: 'Open gallery tab',
		scope: 'project-review',
		action: (ctx) => ctx.openTab('gallery'),
	},
	'review-tab-disclosures': {
		keybind: 'S',
		description: 'Open disclosures tab',
		scope: 'project-review',
		action: (ctx) => ctx.openTab('disclosures'),
	},
	'review-tab-versions': {
		keybind: 'V',
		description: 'Open versions tab',
		scope: 'project-review',
		action: (ctx) => ctx.openTab('versions'),
	},
	'review-tab-permissions': {
		keybind: 'P',
		description: 'Open permissions tab',
		scope: 'project-review',
		action: (ctx) => ctx.openTab('permissions'),
	},
	'review-tab-tech-review': {
		keybind: 'T',
		description: 'Open tech review tab',
		scope: 'project-review',
		action: (ctx) => ctx.openTab('tech-review'),
	},
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
		reviewAction: 'collapse',
		keybind: 'Shift+C',
		description: 'Toggle collapse/expand',
		scope: 'checklist',
		action: (ctx) => ctx.actions.tryToggleCollapse(),
	},
	'reset-progress': {
		reviewAction: 'reset',
		keybind: 'Ctrl+Shift+R',
		description: 'Reset moderation progress',
		scope: 'checklist',
		action: (ctx) => ctx.actions.tryResetProgress(),
	},
	'skip-project': {
		reviewAction: 'next',
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
		description: isOfficialModrinthHost()
			? 'Open current page on production/staging'
			: 'Open current page on alternative host',
		scope: 'global',
		enabled: (ctx) => !isOfficialModrinthHost() || !!ctx.alternativeUrl,
		action: (ctx) => {
			const url = isOfficialModrinthHost() ? ctx.alternativeUrl : ctx.officialUrl
			globalThis.open(url, '_blank', 'noopener,noreferrer')
		},
	},
	'copy-official-site': {
		keybind: 'Ctrl+Shift+O',
		description: 'Copy official URL',
		scope: 'global',
		enabled: () => !isOfficialModrinthHost(),
		action: async (ctx) => {
			await navigator.clipboard.writeText(ctx.officialUrl)
			const environment = ctx.officialUrl.startsWith('https://staging.modrinth.com')
				? 'staging'
				: 'production'
			ctx.notifyCopied(ctx.officialUrl, `Copied ${environment} URL to clipboard`)
		},
	},
	'approve-project': {
		reviewAction: 'approve',
		keybind: 'Shift+Alt+A',
		description: 'Approve project',
		scope: 'checklist',
		action: (ctx) => ctx.actions.tryApprove(),
	},
	'withhold-project': {
		reviewAction: 'withhold',
		keybind: 'Shift+Alt+W',
		description: 'Withhold project',
		scope: 'checklist',
		action: (ctx) => ctx.actions.tryWithhold(),
	},
	'reject-project': {
		reviewAction: 'reject',
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
