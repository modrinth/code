document.addEventListener(
  'click',
  function (e) {
    window.top.postMessage({ modrinthAdClick: true }, 'https://bbsmc.net')

    let target = e.target
    while (target != null) {
      if (target.matches('a')) {
        e.preventDefault()
        if (target.href) {
          window.top.postMessage({ modrinthOpenUrl: target.href }, 'https://bbsmc.net')
        }
        break
      }
      target = target.parentElement
    }
  },
  true,
)

window.open = (url, target, features) => {
  window.top.postMessage({ modrinthOpenUrl: url }, 'https://bbsmc.net')
}
