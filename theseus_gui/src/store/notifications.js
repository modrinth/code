import { defineStore } from 'pinia'

export const useNotifications = defineStore('notificationStore', {
  state: () => ({
    notifications: [], // should be an array of objects. { id, title, text, type, timer? }
  }),
  actions: {
    setNotificationTimer(notif) {
      if (!notif) return

      if (notif.timer) clearTimeout(notif.timer)

      notif.timer = setTimeout(() => {
        this.clearNotificationByIndex(this.notifications.indexOf(notif))
      }, 30000)
    },
    addNotification(newNotif) {
      const existingNotif = this.notifications.find(
        (n) =>
          n.text === newNotif.text &&
          n.title === newNotif.title &&
          n.type === this.notifications.type
      )

      if (existingNotif) {
        this.setNotificationTimer(existingNotif)
        return
      }

      newNotif.id = new Date()
      this.setNotificationTimer(newNotif)
      this.notifications.push(newNotif)
    },
    clearNotificationByIndex(index) {
      this.notifications.splice(index, 1)
    },
    clearNotificationById(id) {
      const index = this.notifications.find((n) => n.id === id)
      this.clearNotificationByIndex(index)
    },
    clearAllNotifications() {
      this.notifications = []
    },
  },
})
