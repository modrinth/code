import type { User } from '@modrinth/ui'

export const getUserLink = (user: User) => {
	return `/user/${user.username}`
}

export const isStaff = (user: User) => {
	return user && STAFF_ROLES.includes(user.role)
}

export const isAdmin = (user: User) => {
	return user && user.role === 'admin'
}

export const STAFF_ROLES = ['moderator', 'admin']
