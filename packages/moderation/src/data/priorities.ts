import { Priority } from '../types/priority'

//TODO: chyz
//TODO: coolbot do this
export const Priorities = new (class extends Priority {
	alerts = this.before()
	rules = this.before()
	rejected = this.before()
	withheld = this.before()
	note = this.after()
})()
