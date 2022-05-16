export default function (context) {
  // Temporary disable analytics
  /*
  if (process.client && context.from.path === context.route.path) {
    return
  }

  if (context.$config.analytics.base_url == null) {
    return
  }

  setTimeout(() => {
    context.$axios
      .post(`${context.$config.analytics.base_url}/register/visit`, {
        path: context.route.path,
        domain: process.server ? context.req.headers.host : location.host,
        consent: false,
      })
      .then(() => {})
      .catch((e) => {
        console.error('An error occurred while registering the visit: ', e)
      })
  })
  */
}
