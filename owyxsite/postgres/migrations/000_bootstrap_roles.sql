-- Bootstrap roles expected by postgres/init.sql dump (OWNER TO owyx / root).
DO $$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'owyx') THEN
    CREATE ROLE owyx LOGIN;
  END IF;
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'root') THEN
    CREATE ROLE root LOGIN;
  END IF;
END
$$;
