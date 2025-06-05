import { pyroFetch } from "../pyro-fetch";
import { ServerModule } from "./base";
import type { Startup, JDKVersion, JDKBuild } from "@modrinth/utils";

export class StartupModule extends ServerModule implements Startup {
  invocation!: string;
  original_invocation!: string;
  jdk_version!: "lts8" | "lts11" | "lts17" | "lts21";
  jdk_build!: "corretto" | "temurin" | "graal";

  async fetch(): Promise<void> {
    const data = await pyroFetch<Startup>(`servers/${this.serverId}/startup`, {}, "startup");
    Object.assign(this, data);
  }

  async update(invocation: string, jdkVersion: JDKVersion, jdkBuild: JDKBuild): Promise<void> {
    await pyroFetch(`servers/${this.serverId}/startup`, {
      method: "POST",
      body: {
        invocation: invocation || null,
        jdk_version: jdkVersion || null,
        jdk_build: jdkBuild || null,
      },
    });
  }
}
