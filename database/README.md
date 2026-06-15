# Database

PostgreSQL migration dosyalari burada tutulur.

Migration akisi Rust API tarafindaki `sqlx::migrate!` ile calisir. Bu nedenle Docker Compose PostgreSQL container'ina migration dosyalari dogrudan mount edilmez; migration kaydi ve tekrar calisma kontrolu uygulama tarafinda tutulur.
