export default defineNitroPlugin((nitroApp) => {
  nitroApp.hooks.hook('render:html', (html, { event }) => {
    const cookies = parseCookies(event)

    if (cookies && cookies['color-mode']) {
      const theme = JSON.parse(cookies['color-mode'])

      html.htmlAttrs.push(`class="${theme.value}-mode"`)
    } else {
      html.htmlAttrs.push(`class="dark-mode"`)
    }
  })
})
