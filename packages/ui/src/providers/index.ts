/**
 * MIT License
 *
 * Copyright (c) 2023 UnoVue
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 *
 * @source https://github.com/unovue/reka-ui/blob/53b4734734f8ebef9a344b1e62db291177c59bfe/packages/core/src/shared/createContext.ts
 */

import type { InjectionKey } from 'vue'
import { inject, provide } from 'vue'

/**
 * @param providerComponentName - The name(s) of the component(s) providing the context.
 *
 * There are situations where context can come from multiple components. In such cases, you might need to give an array of component names to provide your context, instead of just a single string.
 *
 * @param contextName The description for injection key symbol.
 */
export function createContext<ContextValue>(
	providerComponentName: string | string[],
	contextName?: string,
) {
	const symbolDescription =
		typeof providerComponentName === 'string' && !contextName
			? `${providerComponentName}Context`
			: contextName

	const injectionKey: InjectionKey<ContextValue | null> = Symbol(symbolDescription)

	/**
	 * @param fallback The context value to return if the injection fails.
	 *
	 * @throws When context injection failed and no fallback is specified.
	 * This happens when the component injecting the context is not a child of the root component providing the context.
	 */
	const injectContext = <T extends ContextValue | null | undefined = ContextValue>(
		fallback?: T,
	): T extends null ? ContextValue | null : ContextValue => {
		const context = inject(injectionKey, fallback)
		if (context) return context

		if (context === null)
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			return context as any

		throw new Error(
			`Injection \`${injectionKey.toString()}\` not found. Component must be used within ${
				Array.isArray(providerComponentName)
					? `one of the following components: ${providerComponentName.join(', ')}`
					: `\`${providerComponentName}\``
			}`,
		)
	}

	const provideContext = (contextValue: ContextValue) => {
		provide(injectionKey, contextValue)
		return contextValue
	}

	return [injectContext, provideContext] as const
}

export * from './api-client'
export * from './page-context'
export * from './project-page'
export * from './server-context'
export * from './web-notifications'
