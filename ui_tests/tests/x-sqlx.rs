use std::collections::HashMap;

fn main() {}

#[async_trait::async_trait]
trait Store {
    async fn put(&mut self, key: String, value: String);
    async fn get(&self, key: String) -> Option<String>;
}

struct InMemoryStore {
    internal: HashMap<String, String>,
}
impl InMemoryStore {
    fn new() -> Self {
        Self {
            internal: HashMap::new(),
        }
    }
}
#[async_trait::async_trait]
impl Store for InMemoryStore {
    async fn put(&mut self, key: String, value: String) {
        self.internal.insert(key, value);
    }

    async fn get(&self, key: String) -> Option<String> {
        self.internal.get(&key).cloned()
    }
}

struct SqliteStore {
    pool: sqlx::Pool<sqlx::sqlite::Sqlite>,
}
impl SqliteStore {
    async fn new(pool: sqlx::Pool<sqlx::sqlite::Sqlite>) -> Self {
        sqlx::query("CREATE TABLE IF NOT EXISTS store ( key TEXT, value TEXT )")
            .execute(&pool)
            .await
            .unwrap();
        Self { pool }
    }
}
#[async_trait::async_trait]
impl Store for SqliteStore {
    async fn put(&mut self, key: String, value: String) {
        sqlx::query("INSERT INTO store (key, value) VALUES (?, ?)")
            .bind(key)
            .bind(value)
            .execute(&self.pool)
            .await
            .unwrap();
    }

    async fn get(&self, key: String) -> Option<String> {
        let result = sqlx::query_as::<_, (String,)>("SELECT value FROM store WHERE key = ?")
            .bind(key)
            .fetch_one(&self.pool)
            .await
            .ok();
        result.map(|(v,)| v)
    }
}

#[flex_mod::flex_mod(define_store_test_suite; constructions(STORE), attribute_substitutions(TEST))]
mod __ {
    // FIXME: `cargo test` thinks this is unused. See test `2_1` for details.
    #[allow(unused_imports)]
    use crate::Store;

    #[__CONSTRUCT(mut store as STORE)]
    #[__SUBSTITUTE(TEST)]
    async fn it_works() {
        assert_eq!(store.get("foo".to_string()).await, None);
        store.put("foo".to_string(), "bar".to_string()).await;
        assert_eq!(store.get("foo".to_string()).await, Some("bar".to_string()));
    }
}

define_store_test_suite! {
    mod in_memory_store_test_suite;
    constructions {
        STORE => crate::InMemoryStore::new(),
    },
    attribute_substitutions {
        TEST => #[::tokio::test],
    },
}

define_store_test_suite! {
    mod sqlite_store_test_suite;
    constructions {
        STORE => crate::SqliteStore::new(pool.clone()).await,
    },
    attribute_substitutions {
        TEST => #[::sqlx::test] (.., pool: ::sqlx::Pool<sqlx::sqlite::Sqlite>),
    },
}
