import type { Schedule, ServerSchedule } from "@modrinth/utils";
import { useServersFetch } from "../servers-fetch.ts";
import { ServerModule } from "./base.ts";

export class SchedulingModule extends ServerModule {
  tasks: ServerSchedule[] = [];

  private optimisticUpdate(action: () => void): () => void {
    const originalTasks = [...this.tasks];
    action();
    return () => {
      this.tasks = originalTasks;
    };
  }

  async fetch(): Promise<void> {
    const response = await useServersFetch<{ schedules: { quota: 32; items: ServerSchedule[] } }>(
      `servers/${this.serverId}/options`,
      { version: 1 },
    );
    this.tasks = response.schedules.items;
  }

  async deleteTask(task: ServerSchedule): Promise<void> {
    const rollback = this.optimisticUpdate(() => {
      this.tasks = this.tasks.filter((t) => t.id !== task.id);
    });

    try {
      await useServersFetch(`servers/${this.serverId}/options/schedules/${task.id}`, {
        method: "DELETE",
        version: 1,
      });
    } catch (error) {
      rollback();
      throw error;
    }
  }

  async createTask(task: Schedule): Promise<number> {
    const rollback = this.optimisticUpdate(() => {});

    try {
      const response = await useServersFetch<{ id: number }>(
        `servers/${this.serverId}/options/schedules`,
        {
          method: "POST",
          body: task,
          version: 1,
        },
      );

      this.tasks.push({ ...task, id: response.id } as ServerSchedule);
      return response.id;
    } catch (error) {
      rollback();
      throw error;
    }
  }

  async editTask(taskId: number, updatedTask: Partial<Schedule>): Promise<void> {
    const rollback = this.optimisticUpdate(() => {
      const taskIndex = this.tasks.findIndex((t) => t.id === taskId);
      if (taskIndex !== -1) {
        this.tasks[taskIndex] = { ...this.tasks[taskIndex], ...updatedTask };
      }
    });

    try {
      await useServersFetch(`servers/${this.serverId}/options/schedules/${taskId}`, {
        method: "PATCH",
        body: updatedTask,
        version: 1,
      });
    } catch (error) {
      rollback();
      throw error;
    }
  }
}
