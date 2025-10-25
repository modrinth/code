DO $$
DECLARE
    admin_user_id BIGINT;
    payout_id BIGINT;
    awarded_amount NUMERIC := 1000.0;
    withdrawn_amount NUMERIC := 500.0;
    withdrawal_fee NUMERIC := 0.0;
BEGIN
    SELECT id INTO admin_user_id FROM users WHERE username = 'Default admin user';

    IF admin_user_id IS NULL THEN
        RAISE EXCEPTION 'Default admin user not found';
    END IF;

    SELECT COALESCE(MAX(id) + 1, 1) INTO payout_id FROM payouts;

    INSERT INTO payouts_values (user_id, mod_id, amount, created, date_available)
    VALUES (
        admin_user_id,
        NULL,
        awarded_amount,
        NOW() - INTERVAL '30 days',
        NOW() - INTERVAL '1 day'
    );

    INSERT INTO payouts (id, user_id, amount, created, status, method, method_address, platform_id, fee)
    VALUES (
        payout_id,
        admin_user_id,
        withdrawn_amount,
        NOW() - INTERVAL '10 days',
        'success',
        'paypal',
        'admin@modrinth.invalid',
        'fixture_payout_' || payout_id,
        withdrawal_fee
    );

    RAISE NOTICE 'Payout fixture applied successfully';
    RAISE NOTICE 'Available balance: % (% awarded - % withdrawn - % fees)',
        awarded_amount - withdrawn_amount - withdrawal_fee,
        awarded_amount,
        withdrawn_amount,
        withdrawal_fee;
    RAISE NOTICE 'Withdrawn YTD: %', withdrawn_amount;
END $$;
