import { defineStore } from 'pinia'

export const useNotifications = defineStore('notificationsStore', {
  state: () => ({
    notificationsWrapper: null,
  }),
  actions: {
    setNotifs(notifs: any) {
      this.notificationsWrapper = notifs
    },
    addNotification(notif: any) {
      ;(this.notificationsWrapper as any).addNotification(notif)
    },
  },
})

export const handleError = (err: any) => {
  const notifs = useNotifications()

  notifs.addNotification({
    title: 'An error occurred',
    text: err.message ?? err,
    type: 'error',
  })

  console.error(err)
}
