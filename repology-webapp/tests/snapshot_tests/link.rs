// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use insta::assert_snapshot;
use sqlx::PgPool;

use repology_webapp_test_utils::Request;

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_nonexistent(pool: PgPool) {
    let response = Request::new(pool, "/link/https://example.com/nonexistent").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_not_checked(pool: PgPool) {
    let response = Request::new(pool, "/link/https://example.com/not-checked").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ipv4_failure(pool: PgPool) {
    let response = Request::new(pool, "/link/https://example.com/ipv4-failure").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ipv4_success(pool: PgPool) {
    let response = Request::new(pool, "/link/https://example.com/ipv4-success").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ipv4_redirect(pool: PgPool) {
    let response = Request::new(pool, "/link/https://example.com/ipv4-redirect").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ipv6_failure(pool: PgPool) {
    let response = Request::new(pool, "/link/https://example.com/ipv6-failure").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ipv6_success(pool: PgPool) {
    let response = Request::new(pool, "/link/https://example.com/ipv6-success").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ipv6_redirect(pool: PgPool) {
    let response = Request::new(pool, "/link/https://example.com/ipv6-redirect").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}

#[ignore]
#[sqlx::test(migrator = "repology_common::MIGRATOR", fixtures("link_data"))]
async fn test_ssl_failure(pool: PgPool) {
    let response = Request::new(pool, "/link/https://example.com/ssl-failure").perform().await;
    assert_snapshot!(response.as_snapshot().unwrap());
}
