export default defineNuxtRouteMiddleware((to) => {
  if (to.query.launcher) {
    setPageLayout("empty");
  }
});
