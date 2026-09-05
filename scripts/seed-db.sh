#!/usr/bin/env bash

source .env
psql $DATABASE_URL < fixtures/labrinth-seed-data-202508052143.sql
