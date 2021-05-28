import axios from 'axios'
export default function (context) {
  if (context.$config.analytics.base_url == null) {
    return
  }
  let domain = ''
  if (process.server) {
    domain = context.req.headers.host
  } else {
    domain = location.host
  }
  const url = context.$config.analytics.base_url + '/register/visit'
  const path = context.route.path.split('?')[0]
  setTimeout(() => {
    axios
      .post(url, {
        path,
        domain,
        consent: false,
      })
      .then(() => {})
      .catch((e) => {
        console.error('An error occurred while registering the visit: ', e)
      })
  })
}
