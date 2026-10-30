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

const panelToggleKeybinds = [
	'1',
	'2',
	'3',
	'4',
	'5',
	'6',
	'7',
	'8',
	'9',
	'Shift+1',
	'Shift+2',
	'Shift+3',
	'Shift+4',
	'Shift+5',
	'Shift+6',
	'Shift+7',
	'Shift+8',
	'Shift+9',
	'Alt+1',
	'Alt+2',
	'Alt+3',
	'Alt+4',
	'Alt+5',
	'Alt+6',
	'Alt+7',
	'Alt+8',
	'Alt+9',
]

const keybinds: { [id: string]: KeybindListener } = {
	...Object.fromEntries(
		panelToggleKeybinds.map((keybind, index) => [
			`review-panel-toggle-${index + 1}`,
			{
				keybind,
				description: `Toggle panel item ${index + 1}`,
				scope: 'review-panel-toggle',
				enabled: (ctx) => ctx.available(index),
				action: (ctx) => ctx.toggle(index),
			} satisfies KeybindListener,
		]),
	),
	'review-toggle-icon': {
		keybind: 'I',
		description: 'Toggle project icon viewer',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('toggle-icon'),
		action: (ctx) => ctx.run('toggle-icon'),
	},
	'review-cycle-conversation': {
		keybind: 'Shift+R',
		description: 'Cycle issues, thread, and re-review',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('cycle-conversation'),
		action: (ctx) => ctx.run('cycle-conversation'),
	},
	'review-cycle-misc': {
		keybind: 'Shift+M',
		description: 'Cycle misc review panels',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('cycle-misc'),
		action: (ctx) => ctx.run('cycle-misc'),
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
	'review-tab-slicer': {
		keybind: 'L',
		description: 'Open slicer tab',
		scope: 'project-review',
		action: (ctx) => ctx.openTab('slicer'),
	},
	'toggle-collapse': {
		keybind: 'Shift+C',
		description: 'Expand or collapse the hovered or focused item',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('collapse'),
		action: (ctx) => ctx.run('collapse'),
	},
	'reset-progress': {
		keybind: 'Ctrl+Shift+R',
		description: 'Open reset issues and message customizations dialog',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('reset'),
		action: (ctx) => ctx.run('reset'),
	},
	'skip-project': {
		keybind: 'Ctrl+Shift+S',
		description: 'Skip/next project',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('next'),
		action: (ctx) => ctx.run('next'),
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
		description: 'Copy permalink on the current host',
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
		description: 'Copy permalink with page on the current host',
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
			? 'Open current page on alternative host'
			: 'Open current page on production/staging',
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
		keybind: 'Shift+Alt+A',
		description: 'Approve project',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('approve'),
		action: (ctx) => ctx.run('approve'),
	},
	'withhold-project': {
		keybind: 'Shift+Alt+W',
		description: 'Withhold project',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('withhold'),
		action: (ctx) => ctx.run('withhold'),
	},
	'reject-project': {
		keybind: 'Shift+Alt+R',
		description: 'Reject project',
		scope: 'review-actions',
		enabled: (ctx) => ctx.available('reject'),
		action: (ctx) => ctx.run('reject'),
	},
	'tech-review-top': {
		keybind: 'ArrowUp',
		description: 'Scroll to the top of the technical review card or detail page',
		scope: 'tech-review',
		action: (ctx) => ctx.actions.goToTop(),
	},
	'tech-review-bottom': {
		keybind: 'ArrowDown',
		description: 'Scroll to the bottom of the technical review card or detail page',
		scope: 'tech-review',
		action: (ctx) => ctx.actions.goToBottom(),
	},
}

export default keybinds
