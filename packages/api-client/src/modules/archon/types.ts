import * as ServersV0 from './servers/types/v0'
import * as ServersV1 from './servers/types/v1'

export namespace Archon {
	export namespace Servers {
		export import v0 = ServersV0
		export import v1 = ServersV1
	}
}
