export const useNotifications = () => useState('notifications', () => [])

export const addNotification = (notification) => {
  const notifications = useNotifications()

  const existingNotif = notifications.value.find(
    (x) =>
      x.text === notification.text && x.title === notification.title && x.type === notification.type
  )
  if (existingNotif) {
    setNotificationTimer(existingNotif)

    return
  }

  notification.id = new Date()

  setNotificationTimer(notification)
  notifications.value.push(notification)
}

export const setNotificationTimer = (notification) => {
  if (!notification) return

  const notifications = useNotifications()

  if (notification.timer) {
    clearTimeout(notification.timer)
  }

  notification.timer = setTimeout(() => {
    notifications.value.splice(notifications.value.indexOf(notification), 1)
  }, 30000)
}
