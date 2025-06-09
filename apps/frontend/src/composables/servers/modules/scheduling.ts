import type { ScheduledTask } from "@modrinth/utils";
import { useServersFetch } from "../servers-fetch.ts";
import { ServerModule } from "./base.ts";

export class SchedulingModule extends ServerModule {
  tasks: ScheduledTask[] = [];

  async fetch(): Promise<void> {
    this.tasks = await useServersFetch<ScheduledTask[]>(
      `servers/${this.serverId}/options/schedules`,
      { version: 1 },
    );
  }

  async deleteTask(task: ScheduledTask): Promise<void> {
    await useServersFetch(`servers/${this.serverId}/options/schedules`, {
      method: "DELETE",
      body: { title: task.title },
      version: 1,
    });
    this.tasks = this.tasks.filter((t) => t.title !== task.title);
  }

  async createTask(task: ScheduledTask): Promise<number> {
    await useServersFetch(`servers/${this.serverId}/options/schedules`, {
      method: "POST",
      body: task,
      version: 1,
    });
    this.tasks.push(task);
    return this.tasks.length;
  }

  async editTask(taskTitle: string, updatedTask: Partial<ScheduledTask>): Promise<void> {
    await useServersFetch(`servers/${this.serverId}/options/schedules`, {
      method: "PATCH",
      body: { title: taskTitle, ...updatedTask },
      version: 1,
    });
    const index = this.tasks.findIndex((t) => t.title === taskTitle);
    if (index !== -1) {
      this.tasks[index] = { ...this.tasks[index], ...updatedTask };
    }
  }
}
