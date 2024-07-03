export const getUserLink = (user) => {
  return `/user/${user.username}`
}

export const isStaff = (user) => {
  return user && STAFF_ROLES.includes(user.role)
}

export const STAFF_ROLES = ['moderator', 'admin']
