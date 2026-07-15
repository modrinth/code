import { ListBulletedIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import {
	action,
	group,
	markdown,
	md,
	type MessageFn,
	option,
	stage,
	toggle,
} from '../../types/node'

//TODO: coolbot have fun :3
export default function () {
	const { projectV3: project } = injectProjectPageContext()

	const prohibitedContentHeaderMsg: MessageFn = md(
		'checklist/messages/other-rules/prohibited-content-header',
	)
	const prohibitedContentMsgs: Record<string, MessageFn> = {
		objectionable: md('checklist/messages/other-rules/prohibited-content/objectionable'),
		discriminatory: md('checklist/messages/other-rules/prohibited-content/discriminatory'),
		'ip-infringement': md('checklist/messages/other-rules/prohibited-content/ip-infringement'),
		'legal-rights': md('checklist/messages/other-rules/prohibited-content/legal-rights'),
		'illegal-activity': md('checklist/messages/other-rules/prohibited-content/illegal-activity'),
		harmful: md('checklist/messages/other-rules/prohibited-content/harmful'),
		misleading: md('checklist/messages/other-rules/prohibited-content/misleading'),
		impersonation: md('checklist/messages/other-rules/prohibited-content/impersonation'),
		'false-endorsement': md('checklist/messages/other-rules/prohibited-content/false-endorsement'),
		profanity: md('checklist/messages/other-rules/prohibited-content/profanity'),
		'undisclosed-upload': md(
			'checklist/messages/other-rules/prohibited-content/undisclosed-upload',
		),
		'mojang-bypass': md('checklist/messages/other-rules/prohibited-content/mojang-bypass'),
	}
	const serverSideOptInHeaderMsg: MessageFn = md(
		'checklist/messages/other-rules/server-side-opt-in-header',
	)
	const serverSideOptInMsgs: Record<string, MessageFn> = {
		'x-ray': md('checklist/messages/other-rules/server-side-opt-in/x-ray'),
		'aim-bot': md('checklist/messages/other-rules/server-side-opt-in/aim-bot'),
		movement: md('checklist/messages/other-rules/server-side-opt-in/movement'),
		pvp: md('checklist/messages/other-rules/server-side-opt-in/pvp'),
		'hiding-mods': md('checklist/messages/other-rules/server-side-opt-in/hiding-mods'),
		'item-duplication': md('checklist/messages/other-rules/server-side-opt-in/item-duplication'),
	}

	return stage('other-rules', 'Rule Following')
		.hint('Does this project violate the rules?')
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e35ee711bf080709084f6269835607f',
		)
		.icon(ListBulletedIcon)
		.navigate('/moderation')
		.children(
			group().children(
				toggle('paid_access_server', 'Paid access server')
					.shown(computed(() => !!project.value.minecraft_server))
					.action(action().suggestedStatus('rejected').severity('critical').message()),

				// TODO: chyz, the lists built by these message have empty line gaps.
				toggle('prohibited_content', 'Prohibited Content')
					.action(
						action()
							.suggestedStatus('rejected')
							.severity('critical')
							.message(async (state) => {
								const header = (await prohibitedContentHeaderMsg(state)).trimEnd()
								const selected = state.options
								if (!(selected instanceof Set) || selected.size === 0) return header
								const items = await Promise.all(
									[...selected].map(
										(id) => prohibitedContentMsgs[id]?.(state) ?? Promise.resolve(''),
									),
								)
								return `${header}\n${items.join('\n')}`
							}),
					)
					.children(
						group()
							.multiSelect('options')
							.title('Which Prohibited Content rules does this project violate?')
							.children(
								option('objectionable', 'Objectionable'),
								option('discriminatory', 'Discriminatory or Explicit'),
								option('ip-infringement', 'IP Infringement'),
								option('legal-rights', 'Rights Violation'),
								option('illegal-activity', 'Illegal Activity'),
								option('harmful', 'Harmful or Deceptive'),
								option('misleading', 'Misleading claims'),
								option('impersonation', 'Impersonation'),
								option('false-endorsement', 'False Endorsement'),
								option('profanity', 'Profanity'),
								option('undisclosed-upload', 'Undisclosed Data Upload'),
								option('mojang-bypass', 'Mojang Bypass'),
							),
					),

				toggle('cheat_or_hack_advertising', 'Hacks').action(
					action().suggestedStatus('rejected').severity('critical').message(),
				),

				toggle('server_side_opt_out', 'Opt-out').action(
					action().suggestedStatus('flagged').severity('high').message(),
				),

				toggle('server_side_opt_in', 'Opt-in')
					.action(
						action()
							.suggestedStatus('flagged')
							.severity('high')
							.message(async (state) => {
								const header = (await serverSideOptInHeaderMsg(state)).trimEnd()
								const selected = state.options
								if (!(selected instanceof Set) || selected.size === 0) return header
								const items = await Promise.all(
									[...selected].map(
										(id) => serverSideOptInMsgs[id]?.(state) ?? Promise.resolve(''),
									),
								)
								return `${header}\n${items.join('\n')}`
							}),
					)
					.children(
						group()
							.multiSelect('options')
							.title('Which features require a Server-side Opt-in?')
							.children(
								option('x-ray', 'X-ray'),
								option('aim-bot', 'Aim Assist'),
								option('movement', 'Movement'),
								option('pvp', 'PvP'),
								option('hiding-mods', 'Anti 3.x'),
								option('item-duplication', 'Dupe'),
							),
					),

				toggle('excessive_languages', 'Excessive languages')
					.shown(
						computed(
							() =>
								!!project.value.minecraft_server &&
								!!project.value.minecraft_server?.languages?.length &&
								project.value.minecraft_server.languages.length > 4,
						),
					)
					.action(action().suggestedStatus('flagged').severity('low').message()),

				toggle('rule_breaking_other', 'Other')
					// TODO: chyz, the required asterisk is on a separate line
					.action(
						action()
							.suggestedStatus('rejected')
							.severity('critical')
							.message((state) => ({ MESSAGE: state.message })),
					)
					.children(
						markdown('message').title('Explain how it infringes on content rules.').required(),
					),
			),
		)
}
