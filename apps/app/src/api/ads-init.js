document.addEventListener('click', function (e) {
  let target = e.target
  while (target != null) {
    if (target.matches('a')) {
      e.preventDefault()
      if (target.href) {
        window.top.postMessage({ modrinthOpenUrl: target.href }, 'https://modrinth.com')
      }
      break
    }
    target = target.parentElement
  }
})

window.open = (url, target, features) => {
  window.top.postMessage({ modrinthOpenUrl: url }, 'https://modrinth.com')
}
