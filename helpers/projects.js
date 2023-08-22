export const getProjectTypeForUrl = (type, categories) => {
  return getProjectTypeForUrlShorthand(type, categories)
}

export const getProjectTypeForUrlShorthand = (type, categories, overrideTags) => {
  const tags = overrideTags ?? useTags().value

  if (type === 'mod') {
    const isMod = categories.some((category) => {
      return tags.loaderData.modLoaders.includes(category)
    })

    const isPlugin = categories.some((category) => {
      return tags.loaderData.allPluginLoaders.includes(category)
    })

    const isDataPack = categories.some((category) => {
      return tags.loaderData.dataPackLoaders.includes(category)
    })

    if (isDataPack) {
      return 'datapack'
    } else if (isPlugin) {
      return 'plugin'
    } else if (isMod) {
      return 'mod'
    } else {
      return 'mod'
    }
  } else {
    return type
  }
}

export const getProjectLink = (project) => {
  return `/${getProjectTypeForUrl(project.project_type, project.loaders)}/${
    project.slug ? project.slug : project.id
  }`
}

export const getVersionLink = (project, version) => {
  if (version) {
    return getProjectLink(project) + '/version/' + version.id
  } else {
    return getProjectLink(project)
  }
}

export const isApproved = (project) => {
  return project && APPROVED_PROJECT_STATUSES.includes(project.status)
}

export const isListed = (project) => {
  return project && LISTED_PROJECT_STATUSES.includes(project.status)
}

export const isUnlisted = (project) => {
  return project && UNLISTED_PROJECT_STATUSES.includes(project.status)
}

export const isPrivate = (project) => {
  return project && PRIVATE_PROJECT_STATUSES.includes(project.status)
}

export const isRejected = (project) => {
  return project && REJECTED_PROJECT_STATUSES.includes(project.status)
}

export const isUnderReview = (project) => {
  return project && UNDER_REVIEW_PROJECT_STATUSES.includes(project.status)
}

export const isDraft = (project) => {
  return project && DRAFT_PROJECT_STATUSES.includes(project.status)
}

export const APPROVED_PROJECT_STATUSES = ['approved', 'archived', 'unlisted', 'private']
export const LISTED_PROJECT_STATUSES = ['approved', 'archived']
export const UNLISTED_PROJECT_STATUSES = ['unlisted', 'withheld']
export const PRIVATE_PROJECT_STATUSES = ['private', 'rejected', 'processing']
export const REJECTED_PROJECT_STATUSES = ['rejected', 'withheld']
export const UNDER_REVIEW_PROJECT_STATUSES = ['processing']
export const DRAFT_PROJECT_STATUSES = ['draft']
