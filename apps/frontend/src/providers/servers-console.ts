import { AbstractModrinthServersConsole } from "@modrinth/ui";
import { type Ref } from "vue";
import { useState } from "#app";

export class FrontendModrinthServersConsole extends AbstractModrinthServersConsole {
  readonly output: Ref<string[]>;
  readonly searchQuery: Ref<string>;
  readonly filteredOutput: Ref<string[]>;

  constructor() {
    super();

    this.output = useState<string[]>("modrinth-servers-console-output", () => []);
    this.searchQuery = useState<string>("modrinth-servers-console-search", () => "");
    this.filteredOutput = useState<string[]>("modrinth-servers-console-filtered", () => []);
  }
}
