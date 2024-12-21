import { defineStore } from 'pinia'

export const useNotifications = defineStore('notificationsStore', {
  state: () => ({
    notificationsWrapper: null,
  }),
  actions: {
    setNotifs(notifs) {
      this.notificationsWrapper = notifs
    },
    addNotification(notif) {
      this.notificationsWrapper.addNotification(notif)
    },
  },
})

export const handleError = (err) => {
  const notifs = useNotifications()
  notifs.addNotification({
    title: 'An error occurred',
    text: err.message ?? err,
    type: 'error',
  })
  console.error(err)
}
