type NonEmptyObject<T> = T extends object ? (keyof T extends never ? never : T) : never
type ValidKeys<T> = NonEmptyObject<T> extends infer O ? (O extends object ? keyof O : never) : never
