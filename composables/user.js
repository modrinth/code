export const useUser = async (force = false) => {
  const user = useState('user', () => {})

  if (!user.value || force || (user.value && Date.now() - user.value.lastUpdated > 300000)) {
    user.value = await initUser()
  }

  return user
}

export const initUser = async () => {
  const auth = (await useAuth()).value

  const user = {
    notifications: [],
    follows: [],
    projects: [],
    lastUpdated: 0,
  }

  if (auth.user && auth.user.id) {
    try {
      const [notifications, follows, projects] = await Promise.all([
        useBaseFetch(`user/${auth.user.id}/notifications`, auth.headers),
        useBaseFetch(`user/${auth.user.id}/follows`, auth.headers),
        useBaseFetch(`user/${auth.user.id}/projects`, auth.headers),
      ])

      user.notifications = notifications
      user.follows = follows
      user.projects = projects
      user.lastUpdated = Date.now()
    } catch (err) {
      console.error(err)
    }
  }

  return user
}

export const initUserNotifs = async () => {
  const auth = (await useAuth()).value
  const user = (await useUser()).value

  if (auth.user && auth.user.id) {
    try {
      user.notifications = await useBaseFetch(`user/${auth.user.id}/notifications`, auth.headers)
    } catch (err) {
      console.error(err)
    }
  }
}

export const initUserFollows = async () => {
  const auth = (await useAuth()).value
  const user = (await useUser()).value

  if (auth.user && auth.user.id) {
    try {
      user.follows = await useBaseFetch(`user/${auth.user.id}/follows`, auth.headers)
    } catch (err) {
      console.error(err)
    }
  }
}

export const initUserProjects = async () => {
  const auth = (await useAuth()).value
  const user = (await useUser()).value

  if (auth.user && auth.user.id) {
    try {
      user.projects = await useBaseFetch(`user/${auth.user.id}/projects`, auth.headers)
    } catch (err) {
      console.error(err)
    }
  }
}

export const userFollowProject = async (project) => {
  const auth = (await useAuth()).value
  const user = (await useUser()).value

  user.follows = user.follows.concat(project)

  setTimeout(() => {
    useBaseFetch(`project/${project.id}/follow`, {
      method: 'POST',
      ...auth.headers,
    })
  })
}

export const userUnfollowProject = async (project) => {
  const auth = (await useAuth()).value
  const user = (await useUser()).value

  user.follows = user.follows.filter((x) => x.id !== project.id)

  setTimeout(() => {
    useBaseFetch(`project/${project.id}/follow`, {
      method: 'DELETE',
      ...auth.headers,
    })
  })
}

export const userDeleteNotification = async (id) => {
  const user = (await useUser()).value

  user.notifications = user.notifications.filter((x) => x.id !== id)
}
