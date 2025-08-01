export default defineEventHandler((event) => {
  return sendRedirect(event, '/settings/account')
})
