import axios from 'axios'
export default async function (context) {
  let domain = ''
  if (process.server) {
    domain = context.req.headers.host
  } else {
    domain = location.host
  }
  const url = context.$config.analytics.base_url + '/register/visit'
  const path = context.route.path.split('?')[0]
  try {
    return await axios.post(url, {
      path,
      domain,
      consent: false,
    })
  } catch (e) {
    // Simply silence the issue.
  }
}
