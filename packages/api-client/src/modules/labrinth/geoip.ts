import { AbstractModule } from '../../core/abstract-module'

export class LabrinthGeoIpModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_geoip'
	}

	public async getCountry(): Promise<string | undefined> {
		const trace = await this.client.request<string>('/trace', {
			api: 'https://api.modrinth.com',
			version: 'cdn-cgi',
			method: 'GET',
			skipAuth: true,
		})
		const country = trace
			.split('\n')
			.find((line) => line.startsWith('loc='))
			?.slice(4)
			.toUpperCase()

		return country?.match(/^[A-Z]{2}$/) ? country : undefined
	}
}
