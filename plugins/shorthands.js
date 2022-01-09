export default ({ store }, inject) => {
  inject('user', store.state.user)
  inject('tag', store.state.tag)
}
