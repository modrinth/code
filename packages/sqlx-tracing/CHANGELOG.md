# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/jdrouet/sqlx-tracing/compare/v0.1.0...v0.2.0) - 2025-10-02

### Added

- add attributes to pool
- make sure returned_rows is populated
- trace on pool connections and transactions
- make it work with PoolConnection
- make transaction part compile
- create pool-connection and transaction

### Fixed

- unused import
- create separate builder for sqlite and postgres
- please clippy
- remove unused traits

### Other

- use opentelemetry-testing from registry
- comment the code
- update readme with pool builder
- ensure pool queries are traced
- release v0.1.0

## [0.1.0](https://github.com/jdrouet/sqlx-tracing/releases/tag/v0.1.0) - 2025-09-07

### Other

- configure for auto release
- update cargo.toml
- set versions in dev deps
- add readme
- configure
- check that it works for sqlite and postgres
- simple project
