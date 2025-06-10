import type { Schedule, ServerSchedule } from "@modrinth/utils";
import { useServersFetch } from "../servers-fetch.ts";
import { ServerModule } from "./base.ts";

export class SchedulingModule extends ServerModule {
  tasks: ServerSchedule[] = [];

  async fetch(): Promise<void> {
    const response = await useServersFetch<{ items: ServerSchedule[] }>(
      `servers/${this.serverId}/options/schedules`,
      { version: 1 },
    );
    this.tasks = response.items;
  }

  async deleteTask(task: ServerSchedule): Promise<void> {
    await useServersFetch(`servers/${this.serverId}/options/schedules/${task.id}`, {
      method: "DELETE",
      version: 1,
    });
    this.tasks = this.tasks.filter((t) => t.id !== task.id);
  }

  async createTask(task: Schedule): Promise<number> {
    const response = await useServersFetch<{ id: number }>(
      `servers/${this.serverId}/options/schedules`,
      {
        method: "POST",
        body: task,
        version: 1,
      },
    );
    await this.fetch();
    return response.id;
  }

  async editTask(taskId: number, updatedTask: Partial<Schedule>): Promise<void> {
    await useServersFetch(`servers/${this.serverId}/options/schedules/${taskId}`, {
      method: "PATCH",
      body: updatedTask,
      version: 1,
    });
    await this.fetch();
  }
}
