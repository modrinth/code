import { useNuxtApp } from '#app'

async function getBulk(type, ids, apiVersion = 2) {
  if (ids.length === 0) {
    return []
  }

  const url = `${type}?ids=${encodeURIComponent(JSON.stringify([...new Set(ids)]))}`
  const { data: bulkFetch } = await useAsyncData(url, () => useBaseFetch(url, { apiVersion }))
  return bulkFetch.value
}

export async function fetchNotifications() {
  try {
    const auth = (await useAuth()).value
    const { data: notifications } = await useAsyncData(`user/${auth.user.id}/notifications`, () =>
      useBaseFetch(`user/${auth.user.id}/notifications`)
    )

    const projectIds = []
    const reportIds = []
    const threadIds = []
    const userIds = []
    const versionIds = []
    const organizationIds = []

    for (const notification of notifications.value) {
      if (notification.body) {
        if (notification.body.project_id) {
          projectIds.push(notification.body.project_id)
        }
        if (notification.body.version_id) {
          versionIds.push(notification.body.version_id)
        }
        if (notification.body.report_id) {
          reportIds.push(notification.body.report_id)
        }
        if (notification.body.thread_id) {
          threadIds.push(notification.body.thread_id)
        }
        if (notification.body.invited_by) {
          userIds.push(notification.body.invited_by)
        }
        if (notification.body.organization_id) {
          organizationIds.push(notification.body.organization_id)
        }
      }
    }

    const reports = await getBulk('reports', reportIds)

    for (const report of reports) {
      if (report.item_type === 'project') {
        projectIds.push(report.item_id)
      } else if (report.item_type === 'user') {
        userIds.push(report.item_id)
      } else if (report.item_type === 'version') {
        versionIds.push(report.item_id)
      }
    }

    const versions = await getBulk('versions', versionIds)

    for (const version of versions) {
      projectIds.push(version.project_id)
    }

    const [projects, threads, users, organizations] = await Promise.all([
      getBulk('projects', projectIds),
      getBulk('threads', threadIds),
      getBulk('users', userIds),
      getBulk('organizations', organizationIds, 3),
    ])

    for (const notification of notifications.value) {
      notification.extra_data = {}
      if (notification.body) {
        if (notification.body.project_id) {
          notification.extra_data.project = projects.find(
            (x) => x.id === notification.body.project_id
          )
        }
        if (notification.body.organization_id) {
          notification.extra_data.organization = organizations.find(
            (x) => x.id === notification.body.organization_id
          )
        }
        if (notification.body.report_id) {
          notification.extra_data.report = reports.find((x) => x.id === notification.body.report_id)

          const type = notification.extra_data.report.item_type
          if (type === 'project') {
            notification.extra_data.project = projects.find(
              (x) => x.id === notification.extra_data.report.item_id
            )
          } else if (type === 'user') {
            notification.extra_data.user = users.find(
              (x) => x.id === notification.extra_data.report.item_id
            )
          } else if (type === 'version') {
            notification.extra_data.version = versions.find(
              (x) => x.id === notification.extra_data.report.item_id
            )
            notification.extra_data.project = projects.find(
              (x) => x.id === notification.extra_data.version.project_id
            )
          }
        }
        if (notification.body.thread_id) {
          notification.extra_data.thread = threads.find((x) => x.id === notification.body.thread_id)
        }
        if (notification.body.invited_by) {
          notification.extra_data.invited_by = users.find(
            (x) => x.id === notification.body.invited_by
          )
        }
        if (notification.body.version_id) {
          notification.extra_data.version = versions.find(
            (x) => x.id === notification.body.version_id
          )
        }
      }
    }

    return notifications.value
  } catch (error) {
    const app = useNuxtApp()
    app.$notify({
      group: 'main',
      title: 'Error loading notifications',
      text: error.data ? error.data.description : error,
      type: 'error',
    })
  }
  return null
}

export function groupNotifications(notifications, includeRead = false) {
  const grouped = []

  for (const notification of notifications) {
    notification.grouped_notifs = []
  }

  for (const notification of notifications.filter((notif) => includeRead || !notif.read)) {
    // Group notifications of the same thread or project id
    if (notification.body) {
      const index = grouped.findIndex(
        (notif) =>
          ((notif.body.thread_id === notification.body.thread_id && !!notif.body.thread_id) ||
            (notif.body.project_id === notification.body.project_id && !!notif.body.project_id)) &&
          notification.read === notif.read
      )
      const notif = grouped[index]
      if (
        notif &&
        (notification.body.type === 'moderator_message' ||
          notification.body.type === 'project_update')
      ) {
        let groupedNotifs = notif.grouped_notifs
        if (!groupedNotifs) {
          groupedNotifs = []
        }
        groupedNotifs.push(notification)
        grouped[index].grouped_notifs = groupedNotifs
      } else {
        grouped.push(notification)
      }
    } else {
      grouped.push(notification)
    }
  }

  return grouped
}

export async function markAsRead(ids) {
  try {
    await useBaseFetch(`notifications?ids=${JSON.stringify([...new Set(ids)])}`, {
      method: 'PATCH',
    })
    return (notifications) => {
      const newNotifs = notifications
      newNotifs.forEach((notif) => {
        if (ids.includes(notif.id)) {
          notif.read = true
        }
      })
      return newNotifs
    }
  } catch (err) {
    const app = useNuxtApp()
    app.$notify({
      group: 'main',
      title: 'Error marking notification as read',
      text: err.data ? err.data.description : err,
      type: 'error',
    })
    return () => {}
  }
}
