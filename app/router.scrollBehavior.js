export default function (to, from, savedPosition) {
  if (
    from == null ||
    (to.name.startsWith('type-id') && from.name.startsWith('type-id')) ||
    to.name === from.name
  ) {
    return savedPosition
  } else {
    return { x: 0, y: 0 }
  }
}
