## **Roughly the routes needed**

- New type `YearMonth` which is represented as eg `2026-05`
  - So it’s an ISO 8601 year and month, but not day
- GET /\_internal/payouts/history

  ```json
  // response
  Array<{
    payouts_date: string // YearMonth
    days: Array<{
      estimated_revenue_usd: number | null
    }>
    status: open | pending | review | paid
    fees_deducted_usd: number
    variance_adjustment_usd: number
    net_estimated_revenue_usd: number
    creator_net_estimated_revenue_usd: number
    modrinth_net_estimated_revenue_usd: number

    // if status is paid
    actual_revenue_usd: number
    total_external_adjustment_usd: number

    net_actual_revenue_usd: number
    creator_net_actual_revenue_usd: number
    modrinth_net_actual_revenue_usd: number

    // admin authed fields
    started_at: string | null // DateTime<Utc>
    started_by: string | null // user id
    detailed_external_adjustments: Array<{
  	  description: string
  	  amount_usd: number
    }> | null
  }>
  ```

- POST /\_internal/payouts/distribution/start
  - Only allowed to run on payout dates in review - not pending

  ```json
  // body
  {
    payouts_date: string // YearMonth
  	totp_code: string // make sure the backend checks that this user has 2FA set up
    amount_received: number
    adjustments: Array<{
  	  description: string
  	  amount: number
    }>
  }

  // response
  // (same as GET /_internal/payouts/distribution)
  ```

  - We’re going to need to handle processing time + cancel time. Better to not be just in frontend, so maybe some new routes?
  - Only one distribution at a time
  - POST /\_internal/payouts/distribution/cancel
  - GET /\_internal/payouts/distribution
    - gets distribution that’s currently awaiting to go out, and details on it

    ```jsx
    // response
    {
      payouts_date: string // YearMonth
      amount_received: number
      adjustments: Array<{
    	  description: string
    	  amount: number
      }>
      started_at: string // DateTime<Utc>
      started_by: string // user ID
      distributes_at: string // DateTime<Utc>
    }
    ```

- Audit logs notes
  - Payout distribution runs will store who started it and when
  - We don’t store an explicit audit log of, this user attempted to start a run / this user cancelled a run / etc.
- Backend notes
  - When we make `payouts_values` rows, we set `available_at` to null
  - When we return when a payout value is estimated to be available, backend computes it on the fly
    - Take the `created` time, set to end of month, add net 70/75/etc, return that
  - When we do a distribution run, take all the payout values with dates in that year and month, and set their `available_at` to now
  - On the payouts routes (routes which fetch from `payouts_values`), we will leave the `available_at` fields as-is, but add a new `estimated_available_at` which is computed on the fly
    - Frontend will use the following logic for copy:
      - `available_at` is present → this is exactly when it will be (or in our case, was made to be) available. this is not an estimate (also, `estimated_available_at` will not be computed)
      - else, `estimated_available_at` is present → say that this actually is an estimate
