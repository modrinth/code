export const acceptTeamInvite = async (teamId) => {
  await useBaseFetch(`team/${teamId}/join`, {
    apiVersion: 3,
    method: 'POST',
  })
}
export const removeSelfFromTeam = async (teamId) => {
  const auth = await useAuth()
  await removeTeamMember(teamId, auth.value.user.id)
}
export const removeTeamMember = async (teamId, userId) => {
  await useBaseFetch(`team/${teamId}/members/${userId}`, {
    apiVersion: 3,
    method: 'DELETE',
  })
}
