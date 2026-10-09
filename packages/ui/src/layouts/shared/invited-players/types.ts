import { defineMessages } from '#ui/composables/i18n'

export type InvitedPlayerMethod = 'direct' | 'link'

export type InvitedPlayerRow = {
	id: string
	username: string
	avatarUrl?: string
	lastPlayedAt: Date | null
	joinedAt: Date | null
	method: InvitedPlayerMethod
	pending?: boolean
}

export const invitedPlayerMethodMessages = defineMessages({
	direct: { id: 'servers.play.players.method.direct', defaultMessage: 'Direct invite' },
	link: { id: 'servers.play.players.method.link', defaultMessage: 'Share link' },
})
