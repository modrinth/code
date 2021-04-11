import Vue from 'vue';

function isAnalyticsOn(ctx) {
  let cookies = null
  if (ctx.req != null) {
    //Server side rendering
    cookies = ctx.req.headers.cookie;
  } else {
    // Rely on the client
    cookies = document.cookie;
  }
  if (cookies == null) return false
  let processed = {}
  cookies.split(';').forEach((e) => {
    let val = e.trim().split('=');
    processed[val[0]] = decodeURI(val[1]);
  })
  let scopes = decodeURIComponent(processed['modrinth-scopes']).split(",");
  return (scopes !== null && scopes.includes('analytics'));
}

export default async function (ctx, inject) {

  const { app } = ctx;
  const config = ctx.$config && ctx.$config.analytics || {};

  const url = config.script_url ?? '<%= options.script_url %>';
  const tag = config.tracking_code ?? '<%= options.tracking_code %>';
  const enabled = config.enabled ?? <%= options.enabled  || false %>;
  // Check if the parameters are not changed by runtime config:



  const UNAMI_LIB_TAG_ID = '<%= options.UNAMI_LIB_TAG_ID %>';


  if (!enabled) {
    return;
  }

  const injectScript = (script) => {
    const scriptIndex = ctx.app.head.script.findIndex(s => s.hid === script.hid);
    if (scriptIndex !== -1) {
      ctx.app.head.script[scriptIndex] = script;
    } else {
      ctx.app.head.script.push(script);
    }
  };
  if (isAnalyticsOn(ctx)) {
    // Inject unami
    const analyticsScript = {
      hid: UNAMI_LIB_TAG_ID,
      src: url,
      'data-website-id': 'c37613de-245d-4767-90e7-ba7980a4f1a2',
      async: true,
      defer: true,
    };
    injectScript(analyticsScript);
  } else {
    // console.log("Analytics scope was denied.")
  }

}

