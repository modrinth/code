import { ListBulletedIcon } from '@modrinth/assets'

import { button, chips, group, markdown, mdMsg, stage, toggle } from '../../types/node'

export default stage(
	'rule-following',
	'Rule Following',
	'Does this project violate the rules?',
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e35ee711bf080709084f6269835607f',
	{
		icon: ListBulletedIcon,
		navigate: '/moderation',
	},
	[
		group().children(
			button('paid_access_server', 'Paid access server')
				.shown(({ project }) => !!project.minecraft_server)
				.weight(0)
				.suggestedStatus('rejected')
				.severity('critical')
				.message(mdMsg('paid-access-server')),

			button('prohibited_content', 'Prohibited Content')
				.weight(100)
				.suggestedStatus('rejected')
				.severity('critical')
				.message(async (ctx) => {
					const header = (await mdMsg('rule-breaking/prohibited-content-header')(ctx)).trimEnd()
					const selected = ctx.state.options
					if (!(selected instanceof Set) || selected.size === 0) return header
					const items = await Promise.all(
						[...selected].map((id) => mdMsg(`rule-breaking/prohibited-content/${id}`)(ctx)),
					)
					return `${header}\n${items.join('\n')}`
				})
				.children(
					chips('options', 'Which Prohibited Content rules does this project violate?').children(
						toggle('objectionable', 'Objectionable'),
						toggle('discriminatory', 'Discriminatory or Explicit'),
						toggle('ip-infringement', 'IP Infringement'),
						toggle('legal-rights', 'Rights Violation'),
						toggle('illegal-activity', 'Illegal Activity'),
						toggle('harmful', 'Harmful or Deceptive'),
						toggle('misleading', 'Misleading claims'),
						toggle('impersonation', 'Impersonation'),
						toggle('false-endorsement', 'False Endorsement'),
						toggle('profanity', 'Profanity'),
						toggle('undisclosed-upload', 'Undisclosed Data Upload'),
						toggle('mojang-bypass', 'Mojang Bypass'),
					),
				),

			button('cheat_or_hack_advertising', 'Hacks')
				.weight(150)
				.suggestedStatus('rejected')
				.severity('critical')
				.message(mdMsg('rule-breaking/cheat-or-hack-advertising')),

			button('server_side_opt_out', 'Opt-out')
				.weight(200)
				.suggestedStatus('flagged')
				.severity('high')
				.message(mdMsg('rule-breaking/server-side-opt-out')),

			button('server_side_opt_in', 'Opt-in')
				.weight(300)
				.suggestedStatus('flagged')
				.severity('high')
				.message(async (ctx) => {
					const header = (await mdMsg('rule-breaking/server-side-opt-in-header')(ctx)).trimEnd()
					const selected = ctx.state.options
					if (!(selected instanceof Set) || selected.size === 0) return header
					const items = await Promise.all(
						[...selected].map((id) => mdMsg(`rule-breaking/server-side-opt-in/${id}`)(ctx)),
					)
					return `${header}\n${items.join('\n')}`
				})
				.children(
					chips('options', 'Which features require a Server-side Opt-in?').children(
						toggle('x-ray', 'X-ray'),
						toggle('aim-bot', 'Aim Assist'),
						toggle('movement', 'Movement'),
						toggle('pvp', 'PvP'),
						toggle('hiding-mods', 'Anti 3.x'),
						toggle('item-duplication', 'Dupe'),
					),
				),

			button('excessive_languages', 'Excessive languages')
				.shown(({ project }) =>
					!!project.minecraft_server &&
					!!project.minecraft_server?.languages?.length &&
					project.minecraft_server.languages.length > 4,
				)
				.weight(0)
				.suggestedStatus('flagged')
				.severity('low')
				.message(mdMsg('misc-metadata/excessive_languages-server')),

			button('rule_breaking_other', 'Other')
				.weight(0)
				.suggestedStatus('rejected')
				.severity('critical')
				.message(mdMsg('rule-breaking', (ctx) => ({ MESSAGE: ctx.state.message })))
				.children(
					markdown('message', 'Explain how it infringes on content rules.').required(),
				),
		),
	],
)
