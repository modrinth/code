export default async function (context) {
  if (!context.from) {
    if (context.app.$cookies.get('auth-token-reset')) {
      // Only remove the cookie related to the auth, instead of removing everything
      context.app.$cookies.remove('auth-token')
      context.app.$cookies.remove('auth-token-reset')
      return
    }

    if (context.route.query.code) {
      const date = new Date(Date.now() + 30 * 24 * 60 * 60 * 1000) // 30 days
      context.app.$cookies.set('auth-token', context.route.query.code, {
        secure: true,
        sameSite: 'Strict',
        httpOnly: true,
        expires: date,
        path: '/',
      })

      await context.store.dispatch('auth/fetchUser', {
        token: context.route.query.code,
      })
    } else if (context.app.$cookies.get('auth-token')) {
      const cookie = context.app.$cookies.get('auth-token')

      await context.store.dispatch('auth/fetchUser', { token: cookie })
    }
  }

  // Disable middleware if options: { auth: false } is set on the route
  if (routeOption(context.route, 'auth', false)) return

  // Disable middleware if no route was matched to allow 404/error page
  if (!getMatchedComponents(context.route, []).length) {
    return
  }

  if (!context.$auth.user) {
    return context.redirect(
      `https://api.modrinth.com/api/v1/auth/init?url=https://modrinth.com${context.route.fullPath}`
    )
  }
}

function routeOption(route, key, value) {
  return route.matched.some((m) => {
    if (process.client) {
      // Client
      return Object.values(m.components).some(
        (component) => component.options && component.options[key] === value
      )
    } else {
      // SSR
      return Object.values(m.components).some((component) =>
        Object.values(component._Ctor).some(
          (ctor) => ctor.options && ctor.options[key] === value
        )
      )
    }
  })
}

function getMatchedComponents(route, matches) {
  return [].concat(
    ...[],
    ...route.matched.map((m, index) => {
      return Object.keys(m.components).map((key) => {
        matches.push(index)
        return m.components[key]
      })
    })
  )
}
