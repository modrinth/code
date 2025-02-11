// noinspection JSUnusedGlobalSymbols

export const getUserLink = (user) => {
  return `/user/${user.username}`
}

export const isStaff = (user) => {
  return user && STAFF_ROLES.includes(user.role)
}

export const isAdmin = (user) => {
  return user && user.role === 'admin'
}

export const STAFF_ROLES = ['moderator', 'admin']
