import { ListBulletedIcon } from '@modrinth/assets'

import { action, toggle, chips, group, markdown, md, stage, check } from '../../types/node'

export default stage('rule-following', 'Rule Following')
	.hint('Does this project violate the rules?')
	.guidance('https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e35ee711bf080709084f6269835607f')
	.icon(ListBulletedIcon)
	.navigate('/moderation')
	.children(
		group().children(
			toggle('paid_access_server', 'Paid access server')
				.shown(({ project }) => !!project.minecraft_server)
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('critical')
						.message(md('checklist/messages/paid-access-server')),
				),

			toggle('prohibited_content', 'Prohibited Content')
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('critical')
						.message(async (ctx) => {
							const header = (await md('checklist/messages/rule-breaking/prohibited-content-header')(ctx)).trimEnd()
							const selected = ctx.state.options
							if (!(selected instanceof Set) || selected.size === 0) return header
							const items = await Promise.all(
								[...selected].map((id) => md(`checklist/messages/rule-breaking/prohibited-content/${id}`)(ctx)),
							)
							return `${header}\n${items.join('\n')}`
						}),
				)
				.children(
					chips('options', 'Which Prohibited Content rules does this project violate?').children(
						check('objectionable', 'Objectionable'),
						check('discriminatory', 'Discriminatory or Explicit'),
						check('ip-infringement', 'IP Infringement'),
						check('legal-rights', 'Rights Violation'),
						check('illegal-activity', 'Illegal Activity'),
						check('harmful', 'Harmful or Deceptive'),
						check('misleading', 'Misleading claims'),
						check('impersonation', 'Impersonation'),
						check('false-endorsement', 'False Endorsement'),
						check('profanity', 'Profanity'),
						check('undisclosed-upload', 'Undisclosed Data Upload'),
						check('mojang-bypass', 'Mojang Bypass'),
					),
				),

			toggle('cheat_or_hack_advertising', 'Hacks')
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('critical')
						.message(md('checklist/messages/rule-breaking/cheat-or-hack-advertising')),
				),

			toggle('server_side_opt_out', 'Opt-out')
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('high')
						.message(md('checklist/messages/rule-breaking/server-side-opt-out')),
				),

			toggle('server_side_opt_in', 'Opt-in')
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('high')
						.message(async (ctx) => {
							const header = (await md('checklist/messages/rule-breaking/server-side-opt-in-header')(ctx)).trimEnd()
							const selected = ctx.state.options
							if (!(selected instanceof Set) || selected.size === 0) return header
							const items = await Promise.all(
								[...selected].map((id) => md(`checklist/messages/rule-breaking/server-side-opt-in/${id}`)(ctx)),
							)
							return `${header}\n${items.join('\n')}`
						}),
				)
				.children(
					chips('options', 'Which features require a Server-side Opt-in?').children(
						check('x-ray', 'X-ray'),
						check('aim-bot', 'Aim Assist'),
						check('movement', 'Movement'),
						check('pvp', 'PvP'),
						check('hiding-mods', 'Anti 3.x'),
						check('item-duplication', 'Dupe'),
					),
				),

			toggle('excessive_languages', 'Excessive languages')
				.shown(({ project }) =>
					!!project.minecraft_server &&
					!!project.minecraft_server?.languages?.length &&
					project.minecraft_server.languages.length > 4,
				)
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('low')
						.message(md('checklist/messages/misc-metadata/excessive_languages-server')),
				),

			toggle('rule_breaking_other', 'Other')
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('critical')
						.message(md('checklist/messages/rule-breaking', (ctx) => ({ MESSAGE: ctx.state.message }))),
				)
				.children(
					markdown('message', 'Explain how it infringes on content rules.').required(),
				),
		),
	)
