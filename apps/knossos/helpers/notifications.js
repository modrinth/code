import { useNuxtApp } from '#imports'

async function getBulk(type, ids, apiVersion = 2) {
  if (ids.length === 0) {
    return []
  }

  const url = `${type}?ids=${encodeURIComponent(JSON.stringify([...new Set(ids)]))}`
  return await useBaseFetch(url, { apiVersion })
}

export async function fetchExtraNotificationData(notifications) {
  const bulk = {
    projects: [],
    reports: [],
    threads: [],
    users: [],
    versions: [],
    organizations: [],
  }

  for (const notification of notifications) {
    if (notification.body) {
      if (notification.body.project_id) {
        bulk.projects.push(notification.body.project_id)
      }
      if (notification.body.version_id) {
        bulk.versions.push(notification.body.version_id)
      }
      if (notification.body.report_id) {
        bulk.reports.push(notification.body.report_id)
      }
      if (notification.body.thread_id) {
        bulk.threads.push(notification.body.thread_id)
      }
      if (notification.body.invited_by) {
        bulk.users.push(notification.body.invited_by)
      }
      if (notification.body.organization_id) {
        bulk.organizations.push(notification.body.organization_id)
      }
    }
  }

  const reports = await getBulk('reports', bulk.reports)
  for (const report of reports) {
    if (report.item_type === 'project') {
      bulk.projects.push(report.item_id)
    } else if (report.item_type === 'user') {
      bulk.users.push(report.item_id)
    } else if (report.item_type === 'version') {
      bulk.versions.push(report.item_id)
    }
  }
  const versions = await getBulk('versions', bulk.versions)
  for (const version of versions) {
    bulk.projects.push(version.project_id)
  }
  const [projects, threads, users, organizations] = await Promise.all([
    getBulk('projects', bulk.projects),
    getBulk('threads', bulk.threads),
    getBulk('users', bulk.users),
    getBulk('organizations', bulk.organizations, 3),
  ])
  for (const notification of notifications) {
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
  return notifications
}

export function groupNotifications(notifications) {
  const grouped = []

  for (let i = 0; i < notifications.length; i++) {
    const current = notifications[i]
    const next = notifications[i + 1]
    if (current.body && i < notifications.length - 1 && isSimilar(current, next)) {
      current.grouped_notifs = [next]

      let j = i + 2
      while (j < notifications.length && isSimilar(current, notifications[j])) {
        current.grouped_notifs.push(notifications[j])
        j++
      }

      grouped.push(current)
      i = j - 1 // skip i to the last ungrouped
    } else {
      grouped.push(current)
    }
  }

  return grouped
}

function isSimilar(notifA, notifB) {
  return !!notifA.body.project_id && notifA.body.project_id === notifB.body.project_id
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
