import { Priority } from '../types/priority'

//TODO: coolbot ask me(chyz) about this
export const Priorities = new class extends Priority {
	rules    = this.before()
	rejected = this.before()
	withheld = this.before()
	note = this.after()
}()
