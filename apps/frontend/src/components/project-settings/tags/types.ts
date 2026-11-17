export interface Category {
	name: string
	header: string
	icon?: string
	project_type: string
}

export interface CategoryGroup {
	id: string
	title: string
	description?: string
	categories: Category[]
}
