// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-nocheck

export const getCurrency = (userCountry) => {
  const countryCurrency = {
    US: 'USD',
    GB: 'GBP',
    EU: 'EUR',
    AT: 'EUR',
    BE: 'EUR',
    CY: 'EUR',
    EE: 'EUR',
    FI: 'EUR',
    FR: 'EUR',
    DE: 'EUR',
    GR: 'EUR',
    IE: 'EUR',
    IT: 'EUR',
    LV: 'EUR',
    LT: 'EUR',
    LU: 'EUR',
    MT: 'EUR',
    NL: 'EUR',
    PT: 'EUR',
    SK: 'EUR',
    SI: 'EUR',
    RU: 'RUB',
    BR: 'BRL',
    JP: 'JPY',
    ID: 'IDR',
    MY: 'MYR',
    PH: 'PHP',
    TH: 'THB',
    VN: 'VND',
    KR: 'KRW',
    TR: 'TRY',
    UA: 'UAH',
    MX: 'MXN',
    CA: 'CAD',
    NZ: 'NZD',
    NO: 'NOK',
    PL: 'PLN',
    CH: 'CHF',
    LI: 'CHF',
    IN: 'INR',
    CL: 'CLP',
    PE: 'PEN',
    CO: 'COP',
    ZA: 'ZAR',
    HK: 'HKD',
    AR: 'ARS',
    KZ: 'KZT',
    UY: 'UYU',
    CN: 'CNY',
    AU: 'AUD',
    TW: 'TWD',
    SA: 'SAR',
    QA: 'QAR',
  }

  return countryCurrency[userCountry] ?? 'USD'
}

export const formatPrice = (locale, price, currency, trimZeros = false) => {
  let formatter = new Intl.NumberFormat(locale, {
    style: 'currency',
    currency,
  })

  const maxDigits = formatter.resolvedOptions().maximumFractionDigits
  const convertedPrice = price / Math.pow(10, maxDigits)

  let minimumFractionDigits = maxDigits

  if (trimZeros && Number.isInteger(convertedPrice)) {
    minimumFractionDigits = 0
  }

  formatter = new Intl.NumberFormat(locale, {
    style: 'currency',
    currency,
    minimumFractionDigits,
  })
  return formatter.format(convertedPrice)
}

export const calculateSavings = (monthlyPlan, plan, months = 12) => {
  const monthlyAnnualized = monthlyPlan * months

  return Math.floor(((monthlyAnnualized - plan) / monthlyAnnualized) * 100)
}

export const createStripeElements = (stripe, paymentMethods, options) => {
  const styles = getComputedStyle(document.body)

  const elements = stripe.elements({
    appearance: {
      variables: {
        colorPrimary: styles.getPropertyValue('--color-brand'),
        colorBackground: styles.getPropertyValue('--experimental-color-button-bg'),
        colorText: styles.getPropertyValue('--color-base'),
        colorTextPlaceholder: styles.getPropertyValue('--color-secondary'),
        colorDanger: styles.getPropertyValue('--color-red'),
        fontFamily: styles.getPropertyValue('--font-standard'),
        spacingUnit: '0.25rem',
        borderRadius: '0.75rem',
      },
    },
    loader: 'never',
    ...options,
  })

  const paymentElement = elements.create('payment')
  paymentElement.mount('#payment-element')

  const addressElement = elements.create('address', {
    mode: 'billing',
    contacts: paymentMethods
      ? [
          ...new Set(
            paymentMethods.map((x) => ({
              address: x.billing_details.address,
              email: x.billing_details.email,
              name: x.billing_details.name,
            })),
          ),
        ]
      : undefined,
  })
  addressElement.mount('#address-element')

  addressElement.on('change', (e) => {
    if (e.value && e.value.address && e.value.address.country) {
      elements.update({ currency: getCurrency(e.value.address.country).toLowerCase() })
    }
  })

  return { elements, paymentElement, addressElement }
}
