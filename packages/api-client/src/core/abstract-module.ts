import type { AbstractModrinthClient } from './abstract-client'

export abstract class AbstractModule {
	protected client: AbstractModrinthClient

	public constructor(client: AbstractModrinthClient) {
		this.client = client
	}

	/**
	 * Get the module's name, used for error reporting & for module field generation.
	 * @returns Module name
	 */
	public abstract getModuleID(): string
}
