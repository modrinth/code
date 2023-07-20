export const acceptTeamInvite = async (teamId) => {
  await useBaseFetch(`team/${teamId}/join`, {
    method: 'POST',
  })
}
export const removeSelfFromTeam = async (teamId) => {
  const auth = await useAuth()
  await removeTeamMember(teamId, auth.user.id)
}
export const removeTeamMember = async (teamId, userId) => {
  await useBaseFetch(`team/${teamId}/members/${userId}`, {
    method: 'DELETE',
  })
}
