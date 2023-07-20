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
      const [notifications, follows] = await Promise.all([
        useBaseFetch(`user/${auth.user.id}/notifications`),
        useBaseFetch(`user/${auth.user.id}/follows`),
      ])

      user.notifications = notifications
      user.follows = follows
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
      user.notifications = await useBaseFetch(`user/${auth.user.id}/notifications`)
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

export const userDeleteNotification = async (id) => {
  const user = (await useUser()).value

  user.notifications = user.notifications.filter((x) => x.id !== id)
}

export const userDeleteNotifications = async (ids) => {
  const user = (await useUser()).value

  user.notifications = user.notifications.filter((x) => !ids.includes(x.id))
}

export const userReadNotifications = async (ids) => {
  const user = (await useUser()).value

  user.notifications = user.notifications.map((x) => {
    if (ids.includes(x.id)) {
      x.read = true
    }
    return x
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
