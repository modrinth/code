CREATE OR REPLACE FUNCTION from_base62(input VARCHAR)
RETURNS BIGINT AS $$
DECLARE
    base INT := 62;
    chars VARCHAR := '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz';
    result BIGINT := 0;
    i INT;
    char VARCHAR;
    index INT;
BEGIN
    FOR i IN 1..LENGTH(input) LOOP
        char := SUBSTRING(input FROM i FOR 1);
        index := POSITION(char IN chars) - 1;
        IF index < 0 THEN
            RAISE EXCEPTION 'Error: Invalid character in input string';
        END IF;
        result := result * base + index;
    END LOOP;

    RETURN result;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION to_base62(input BIGINT)
RETURNS VARCHAR AS $$
DECLARE
    base INT := 62;
    chars VARCHAR := '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz';
    result VARCHAR := '';
    remainder INT;
BEGIN
    WHILE input > 0 LOOP
        remainder := input % base;
        result := SUBSTRING(chars FROM remainder + 1 FOR 1) || result;
        input := input / base;
    END LOOP;

    RETURN result;
END;
$$ LANGUAGE plpgsql;
