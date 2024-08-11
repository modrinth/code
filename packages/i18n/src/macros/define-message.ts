import { MessageDescriptor } from '@vintl/vintl'
import { parse, ParserOptions as MessageParserOptions } from '@formatjs/icu-messageformat-parser'

type ProperMessageDescriptor<I extends string> = Omit<MessageDescriptor<I>, 'defaultMessage'> & {
  defaultMessage: string
}

const includeDefaultMessage = ['1', 'true'].includes(
  process.env.MODRINTH_I18N_DEFAULT_MESSAGE?.toLowerCase() ?? 'false',
)

if (includeDefaultMessage) {
  console.warn(
    "[defineMessage] Default messages are now processed and included in the processed descriptor. This may significantly increase consumers' bundle sizes due to duplication.",
  )
}

/**
 * A macro that takes in a static descriptor and emits a JS object representing
 * the same descriptor with any fields other than `id` and `defaultMessage`
 * deleted. The `defaultMessage` field is also converted to AST, which serves as
 * a validation that the message syntax is correct, as well as allows the
 * message to be used without bringing compiler to runtime.
 *
 * @param param0 Message descriptor to process.
 * @param opts Options for the message parser.
 * @returns JS object with the processed message descriptor.
 */
export function defineMessage<I extends string>(
  this: unknown,
  { id, defaultMessage }: ProperMessageDescriptor<I>,
  opts?: MessageParserOptions,
): MessageDescriptor<I> {
  if (defaultMessage == null) {
    throw new RangeError(`[defineMessage] ${id} is missing 'defaultMessage'`)
  }

  if (typeof defaultMessage !== 'string') {
    throw new RangeError(`[defineMessage] ${id} 'defaultMessage' must be a string`)
  }

  return includeDefaultMessage ? { id, defaultMessage: parse(defaultMessage, opts) } : { id }
}
