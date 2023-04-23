export const releaseColor = (releaseType) => {
  switch (releaseType) {
    case 'release':
      return 'green'
    case 'beta':
      return 'orange'
    case 'alpha':
      return 'red'
    default:
      return ''
  }
}

export const checkInstalled = (profile, checkProject) => {
  return Object.values(profile.projects).some((p) => p.metadata?.project?.id === checkProject)
}
