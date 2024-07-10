// noinspection JSUnusedGlobalSymbols

export const getProjectTypeForDisplay = (type, categories, tags) => {
  if (type === 'mod') {
    const isPlugin = categories.some((category) => {
      return tags.loaderData.allPluginLoaders.includes(category)
    })
    const isMod = categories.some((category) => {
      return tags.loaderData.modLoaders.includes(category)
    })
    const isDataPack = categories.some((category) => {
      return tags.loaderData.dataPackLoaders.includes(category)
    })

    if (isMod && isPlugin && isDataPack) {
      return 'mod, plugin, and data pack'
    } else if (isMod && isPlugin) {
      return 'mod and plugin'
    } else if (isMod && isDataPack) {
      return 'mod and data pack'
    } else if (isPlugin && isDataPack) {
      return 'plugin and data pack'
    } else if (isDataPack) {
      return 'data pack'
    } else if (isPlugin) {
      return 'plugin'
    }
  }

  return type
}

export const getProjectTypeForUrl = (type, loaders, tags) => {
  if (type === 'mod') {
    const isMod = loaders.some((category) => {
      return tags.loaderData.modLoaders.includes(category)
    })

    const isPlugin = loaders.some((category) => {
      return tags.loaderData.allPluginLoaders.includes(category)
    })

    const isDataPack = loaders.some((category) => {
      return tags.loaderData.dataPackLoaders.includes(category)
    })

    if (isDataPack) {
      return 'datapack'
    } else if (isPlugin) {
      return 'plugin'
    } else if (isMod) {
      return 'mod'
    }
    return 'mod'
  }
  return type
}

export const getProjectLink = (project) => {
  return `/${getProjectTypeForUrl(project.project_type, project.loaders)}/${
    project.slug ? project.slug : project.id
  }`
}

export const getVersionLink = (project, version) => {
  if (version) {
    return `${getProjectLink(project)}/version/${version.id}`
  }
  return getProjectLink(project)
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
