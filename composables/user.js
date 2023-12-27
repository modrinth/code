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
    lastUpdated: 0,
  }

  if (auth.user && auth.user.id) {
    try {
      const [follows, collections] = await Promise.all([
        useBaseFetch(`user/${auth.user.id}/follows`),
        useBaseFetch(`user/${auth.user.id}/collections`, { apiVersion: 3 }),
      ])

      user.collections = collections
      user.follows = follows
      user.lastUpdated = Date.now()
    } catch (err) {
      console.error(err)
    }
  }

  return user
}

export const initUserCollections = async () => {
  const auth = (await useAuth()).value
  const user = (await useUser()).value

  if (auth.user && auth.user.id) {
    try {
      user.collections = await useBaseFetch(`user/${auth.user.id}/collections`, { apiVersion: 3 })
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
      user.follows = await useBaseFetch(`user/${auth.user.id}/follows`)
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
      user.projects = await useBaseFetch(`user/${auth.user.id}/projects`)
    } catch (err) {
      console.error(err)
    }
  }
}

export const userCollectProject = async (collection, projectId) => {
  const user = (await useUser()).value

  const add = !collection.projects.includes(projectId)
  const projects = add
    ? [...collection.projects, projectId]
    : [...collection.projects].filter((x) => x !== projectId)

  const idx = user.collections.findIndex((x) => x.id === collection.id)
  if (idx >= 0) {
    user.collections[idx].projects = projects
  }

  await useBaseFetch(`collection/${collection.id}`, {
    method: 'PATCH',
    body: {
      new_projects: projects,
    },
    apiVersion: 3,
  })
}

export const userFollowProject = async (project) => {
  const user = (await useUser()).value

  user.follows = user.follows.concat(project)
  project.followers++

  setTimeout(() => {
    useBaseFetch(`project/${project.id}/follow`, {
      method: 'POST',
    })
  })
}

export const userUnfollowProject = async (project) => {
  const user = (await useUser()).value

  user.follows = user.follows.filter((x) => x.id !== project.id)
  project.followers--

  setTimeout(() => {
    useBaseFetch(`project/${project.id}/follow`, {
      method: 'DELETE',
    })
  })
}

export const resendVerifyEmail = async () => {
  const app = useNuxtApp()

  startLoading()
  try {
    await useBaseFetch('auth/email/resend_verify', {
      method: 'POST',
    })

    const auth = await useAuth()
    app.$notify({
      group: 'main',
      title: 'Email sent',
      text: `An email with a link to verify your account has been sent to ${auth.value.user.email}.`,
      type: 'success',
    })
  } catch (err) {
    app.$notify({
      group: 'main',
      title: 'An error occurred',
      text: err.data.description,
      type: 'error',
    })
  }
  stopLoading()
}

export const logout = async () => {
  startLoading()
  const auth = await useAuth()
  try {
    await useBaseFetch(`session/${auth.value.token}`, {
      method: 'DELETE',
    })
  } catch {}

  await useAuth('none')
  useCookie('auth-token').value = null
  await navigateTo('/')
  stopLoading()
}
