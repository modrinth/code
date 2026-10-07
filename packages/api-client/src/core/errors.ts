import type { ApiErrorData, ModrinthErrorResponse } from '../types/errors'
import { isModrinthErrorResponse } from '../types/errors'

/**
 * Base error class for all Modrinth API errors
 */
export class ModrinthApiError extends Error {
	/**
	 * HTTP status code (if available)
	 */
	readonly statusCode?: number

	/**
	 * Original error that was caught
	 */
	readonly originalError?: Error

	/**
	 * Response data from the API (if available)
	 */
	readonly responseData?: unknown

	/**
	 * Error context (e.g., module name, operation being performed)
	 */
	readonly context?: string

	/**
	 * Full URL of the request that failed (if available)
	 */
	url?: string

	constructor(message: string, data?: ApiErrorData) {
		super(message)
		this.name = 'ModrinthApiError'

		this.statusCode = data?.statusCode
		this.originalError = data?.originalError
		this.responseData = data?.responseData
		this.context = data?.context

		// Maintains proper stack trace for where our error was thrown (only available on V8)
		if (Error.captureStackTrace) {
			Error.captureStackTrace(this, ModrinthApiError)
		}
	}

	/**
	 * Create a ModrinthApiError from an unknown error
	 */
	static fromUnknown(error: unknown, context?: string): ModrinthApiError {
		if (error instanceof ModrinthApiError) {
			return error
		}

		if (error instanceof Error) {
			return new ModrinthApiError(error.message, {
				originalError: error,
				context,
			})
		}

		return new ModrinthApiError(String(error), { context })
	}
}

/**
 * Error class for Modrinth server errors (kyros/archon)
 * Extends ModrinthApiError with V1 error response parsing
 */
export class ModrinthServerError extends ModrinthApiError {
	/**
	 * V1 error information (if available)
	 */
	readonly v1Error?: ModrinthErrorResponse

	constructor(message: string, data?: ApiErrorData & { v1Error?: ModrinthErrorResponse }) {
		const errorMessage = data?.v1Error ? formatV1ErrorMessage(data.v1Error) : message

		super(errorMessage, data)
		this.name = 'ModrinthServerError'
		this.v1Error = data?.v1Error

		if (Error.captureStackTrace) {
			Error.captureStackTrace(this, ModrinthServerError)
		}
	}

	/**
	 * Create a ModrinthServerError from response data
	 */
	static fromResponse(
		statusCode: number,
		responseData: unknown,
		context?: string,
	): ModrinthServerError {
		const v1Error = isModrinthErrorResponse(responseData) ? responseData : undefined
		const isHtmlPage = typeof responseData === 'string' && /<!doctype html|<html/i.test(responseData)

		const message =
			typeof responseData === 'string' && !isHtmlPage ? responseData : `HTTP ${statusCode}`

		return new ModrinthServerError(message, {
			statusCode,
			responseData: isHtmlPage ? undefined : responseData,
			context,
			v1Error,
		})
	}

	/**
	 * Create a ModrinthServerError from an unknown error
	 */
	static fromUnknown(error: unknown, context?: string): ModrinthServerError {
		if (error instanceof ModrinthServerError) {
			return error
		}

		if (error instanceof ModrinthApiError) {
			return new ModrinthServerError(error.message, {
				statusCode: error.statusCode,
				originalError: error.originalError,
				responseData: error.responseData,
				context: context ?? error.context,
			})
		}

		if (error instanceof Error) {
			return new ModrinthServerError(error.message, {
				originalError: error,
				context,
			})
		}

		return new ModrinthServerError(String(error), { context })
	}
}

function formatV1ErrorMessage(error: ModrinthErrorResponse): string {
	return readableErrorDetail(error.details) ?? error.description
}

function readableErrorDetail(details: unknown): string | undefined {
	if (typeof details === 'string') {
		const trimmed = details.trim()
		return trimmed || undefined
	}

	if (!Array.isArray(details)) {
		return undefined
	}

	return details
		.findLast((detail): detail is string => {
			if (typeof detail !== 'string') {
				return false
			}
			const trimmed = detail.trim()
			return Boolean(trimmed)
		})
		?.trim()
}
