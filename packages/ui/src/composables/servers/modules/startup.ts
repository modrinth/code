import type { Startup, JDKVersion, JDKBuild } from "@modrinth/utils";
import { ServerModule } from "./base.js";

export class StartupModule extends ServerModule implements Startup {
  invocation!: string;
  original_invocation!: string;
  jdk_version!: JDKVersion;
  jdk_build!: JDKBuild;

  async fetch(): Promise<void> {
    const data = await this.server.request<Startup>(`servers/${this.serverId}/startup`, {}, "startup");
    Object.assign(this, data);
  }

  async update(invocation: string, jdkVersion: JDKVersion, jdkBuild: JDKBuild): Promise<void> {
    await this.server.request(`servers/${this.serverId}/startup`, {
      method: "POST",
      body: {
        invocation: invocation || null,
        jdk_version: jdkVersion || null,
        jdk_build: jdkBuild || null,
      },
    });
  }
}
