import { Priority } from '../types/priority'

//TODO: chyz
//TODO: coolbot do this
export const Priorities = new (class extends Priority {
	rules = this.before()
	rejected = this.before()
	withheld = this.before()
	note = this.after()
})()
