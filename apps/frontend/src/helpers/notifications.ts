import { useNuxtApp } from "#imports";

// TODO: There needs to be a standardized way to get these types, eg; @modrinth/types generated from api schema. Later problem.
type Project = { id: string };
type Version = { id: string; project_id: string };
type Report = { id: string; item_type: "project" | "user" | "version"; item_id: string };
type Thread = { id: string };
type User = { id: string };
type Organization = { id: string };

export type NotificationAction = {
  title: string;
  action_route: [string, string];
};

export type NotificationBody = {
  project_id?: string;
  version_id?: string;
  report_id?: string;
  thread_id?: string;
  invited_by?: string;
  organization_id?: string;
};

export type Notification = {
  id: string;
  user_id: string;
  type: "project_update" | "team_invite" | "status_change" | "moderator_message";
  title: string;
  text: string;
  link: string;
  read: boolean;
  created: string;
  actions: NotificationAction[];
  body?: NotificationBody;
  extra_data?: Record<string, unknown>;
  grouped_notifs?: Notification[];
};

async function getBulk<T extends { id: string }>(
  type: string,
  ids: string[],
  apiVersion = 2,
): Promise<T[]> {
  if (!ids || ids.length === 0) {
    return [];
  }
  const url = `${type}?ids=${encodeURIComponent(JSON.stringify([...new Set(ids)]))}`;
  try {
    const res = await useBaseFetch(url, { apiVersion });
    return Array.isArray(res) ? res : [];
  } catch {
    return [];
  }
}

export async function fetchExtraNotificationData(
  notifications: Notification[],
): Promise<Notification[]> {
  const bulk = {
    projects: [] as string[],
    reports: [] as string[],
    threads: [] as string[],
    users: [] as string[],
    versions: [] as string[],
    organizations: [] as string[],
  };

  for (const notification of notifications) {
    if (notification.body) {
      if (notification.body.project_id) bulk.projects.push(notification.body.project_id);
      if (notification.body.version_id) bulk.versions.push(notification.body.version_id);
      if (notification.body.report_id) bulk.reports.push(notification.body.report_id);
      if (notification.body.thread_id) bulk.threads.push(notification.body.thread_id);
      if (notification.body.invited_by) bulk.users.push(notification.body.invited_by);
      if (notification.body.organization_id)
        bulk.organizations.push(notification.body.organization_id);
    }
  }

  const reports = (await getBulk<Report>("reports", bulk.reports)).filter(Boolean);
  for (const r of reports) {
    if (!r?.item_type) continue;
    if (r.item_type === "project") bulk.projects.push(r.item_id);
    else if (r.item_type === "user") bulk.users.push(r.item_id);
    else if (r.item_type === "version") bulk.versions.push(r.item_id);
  }

  const versions = (await getBulk<Version>("versions", bulk.versions)).filter(Boolean);
  for (const v of versions) bulk.projects.push(v.project_id);

  const [projects, threads, users, organizations] = await Promise.all([
    getBulk<Project>("projects", bulk.projects),
    getBulk<Thread>("threads", bulk.threads),
    getBulk<User>("users", bulk.users),
    getBulk<Organization>("organizations", bulk.organizations, 3),
  ]);

  for (const n of notifications) {
    n.extra_data = {};
    if (n.body) {
      if (n.body.project_id)
        n.extra_data.project = projects.find((x) => x.id === n.body!.project_id);
      if (n.body.organization_id)
        n.extra_data.organization = organizations.find((x) => x.id === n.body!.organization_id);
      if (n.body.report_id) {
        n.extra_data.report = reports.find((x) => x.id === n.body!.report_id);
        const t = (n.extra_data.report as Report | undefined)?.item_type;
        if (t === "project")
          n.extra_data.project = projects.find(
            (x) => x.id === (n.extra_data?.report as Report | undefined)?.item_id,
          );
        else if (t === "user")
          n.extra_data.user = users.find(
            (x) => x.id === (n.extra_data?.report as Report | undefined)?.item_id,
          );
        else if (t === "version") {
          n.extra_data.version = versions.find(
            (x) => x.id === (n.extra_data?.report as Report | undefined)?.item_id,
          );
          n.extra_data.project = projects.find(
            (x) => x.id === (n.extra_data?.version as Version | undefined)?.project_id,
          );
        }
      }
      if (n.body.thread_id) n.extra_data.thread = threads.find((x) => x.id === n.body!.thread_id);
      if (n.body.invited_by)
        n.extra_data.invited_by = users.find((x) => x.id === n.body!.invited_by);
      if (n.body.version_id)
        n.extra_data.version = versions.find((x) => x.id === n.body!.version_id);
    }
  }
  return notifications;
}

export function groupNotifications(notifications: Notification[]): Notification[] {
  const grouped: Notification[] = [];
  for (let i = 0; i < notifications.length; i++) {
    const current = notifications[i];
    const next = notifications[i + 1];
    if (current.body && i < notifications.length - 1 && isSimilar(current, next)) {
      current.grouped_notifs = [next];
      let j = i + 2;
      while (j < notifications.length && isSimilar(current, notifications[j])) {
        current.grouped_notifs.push(notifications[j]);
        j++;
      }
      grouped.push(current);
      i = j - 1;
    } else {
      grouped.push(current);
    }
  }
  return grouped;
}

function isSimilar(a: Notification, b: Notification | undefined): boolean {
  return !!a?.body?.project_id && a.body!.project_id === b?.body?.project_id;
}

export async function markAsRead(
  ids: string[],
): Promise<(notifications: Notification[]) => Notification[]> {
  try {
    await useBaseFetch(`notifications?ids=${JSON.stringify([...new Set(ids)])}`, {
      method: "PATCH",
    });
    return (notifications: Notification[]) => {
      const newNotifs = notifications ?? [];
      newNotifs.forEach((n) => {
        if (ids.includes(n.id)) n.read = true;
      });
      return newNotifs;
    };
  } catch (err: any) {
    const app: any = useNuxtApp();
    app.$notify({
      group: "main",
      title: "Error marking notification as read",
      text: err?.data?.description ?? err,
      type: "error",
    });
    return () => [];
  }
}
