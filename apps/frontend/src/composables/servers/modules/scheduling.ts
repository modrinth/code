import type { ScheduledTask } from "@modrinth/utils";
import { useServersFetch } from "../servers-fetch.ts";
import { ServerModule } from "./base.ts";

export class ScheudlingModule extends ServerModule {
  tasks: ScheduledTask[] = [];

  async fetch(): Promise<void> {
    this.tasks = await useServersFetch<ScheduledTask[]>(
      `servers/${this.serverId}/options/schedules`,
      { version: 1 },
    );
  }

  async deleteTask(_task: ScheduledTask): Promise<void> {
    // ...
  }

  async createTask(_task: ScheduledTask): Promise<number> {
    return await 1;
  }

  async editTask(_taskID: number, _task: Partial<ScheduledTask>) {
    // ...
  }
}
