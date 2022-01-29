export default function (to, from, savedPosition) {
  if (to.name.startsWith('type-id') && from.name.startsWith('type-id')) {
    return savedPosition
  } else {
    return { x: 0, y: 0 }
  }
}
