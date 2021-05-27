export default ({ store }, inject) => {
  inject('user', store.state.user)
}
