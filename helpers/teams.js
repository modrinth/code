export const acceptTeamInvite = async (teamId) => {
  const app = useNuxtApp()
  await useBaseFetch(`team/${teamId}/join`, {
    method: 'POST',
    ...app.$defaultHeaders(),
  })
}
export const removeSelfFromTeam = async (teamId) => {
  const app = useNuxtApp()
  await removeTeamMember(teamId, app.$auth.user.id)
}
export const removeTeamMember = async (teamId, userId) => {
  const app = useNuxtApp()
  await useBaseFetch(`team/${teamId}/members/${userId}`, {
    method: 'DELETE',
    ...app.$defaultHeaders(),
  })
}
