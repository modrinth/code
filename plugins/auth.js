export default ({ store }, inject) => {
  inject('auth', store.state.auth)
}
