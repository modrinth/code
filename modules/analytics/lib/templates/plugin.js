// eslint-disable-next-line require-await
export default async function (ctx, inject) {
  const config = (ctx.$config && ctx.$config.analytics) || {}

  const url = config.script_url ?? '<%= options.script_url %>'
  const tag = config.token ?? '<%= options.token %>'
  // eslint-disable-next-line
  const enabled = config.enabled ?? ('<%= options.enabled  || false %>' === 'true');
  // Check if the parameters are not changed by runtime config:

  const UNAMI_LIB_TAG_ID = '<%= options.UNAMI_LIB_TAG_ID %>'

  if (!enabled) {
    return
  }

  const injectScript = (script) => {
    const scriptIndex = ctx.app.head.script.findIndex(
      (s) => s.hid === script.hid
    )
    if (scriptIndex !== -1) {
      ctx.app.head.script[scriptIndex] = script
    } else {
      ctx.app.head.script.push(script)
    }
  }
  const analyticsScript = {
    hid: UNAMI_LIB_TAG_ID,
    src: url,
    'data-cf-beacon': JSON.stringify({
      token: tag,
      spa: true,
    }),
    defer: true,
  }
  injectScript(analyticsScript)
}
