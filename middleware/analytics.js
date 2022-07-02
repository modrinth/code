export default function (context) {
  if (process.client && context.from.path === context.route.path) {
    return
  }

  if (context.$config.analytics.base_url == null) {
    return
  }

  setTimeout(() => {
    context.$axios
      .post(
        `${context.$config.analytics.base_url}view`,
        {
          url: process.env.domain + context.route.fullPath,
        },
        context.$config.analytics.admin_key
          ? {
              headers: {
                'Modrinth-Admin': context.$config.analytics.admin_key,
              },
            }
          : {}
      )
      .then(() => {})
      .catch((e) => {
        console.error('An error occurred while registering the visit: ', e)
      })
  })
}
