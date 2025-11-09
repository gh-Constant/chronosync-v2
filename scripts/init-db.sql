-- scripts/init-db.sql

-- Users table
CREATE TABLE users (
                       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                       email VARCHAR(255) UNIQUE NOT NULL,
                       password_hash VARCHAR(255) NOT NULL,
                       username VARCHAR(100) UNIQUE NOT NULL,
                       created_at TIMESTAMPTZ DEFAULT NOW(),
                       updated_at TIMESTAMPTZ DEFAULT NOW(),
                       timezone VARCHAR(50) DEFAULT 'UTC'
);

-- App/Website categories
CREATE TABLE app_categories (
                                id SERIAL PRIMARY KEY,
                                name VARCHAR(100) UNIQUE NOT NULL,
                                description TEXT
);

-- Applications / Websites
CREATE TABLE applications (
                              id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                              name VARCHAR(255) NOT NULL,
                              category_id INT REFERENCES app_categories(id) ON DELETE SET NULL,
                              created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Usage logs (time-series data)
CREATE TABLE usage_logs (
                            id BIGSERIAL,
                            user_id UUID REFERENCES users(id) ON DELETE CASCADE,
                            app_id UUID REFERENCES applications(id) ON DELETE CASCADE,
                            time TIMESTAMPTZ NOT NULL,  -- ✅ colonne temporelle obligatoire
                            duration_seconds INT NOT NULL,
                            cpu_usage FLOAT,
                            memory_usage FLOAT,
                            created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Convert to TimescaleDB hypertable
SELECT create_hypertable('usage_logs', 'time');

-- Optional indexes for performance
CREATE INDEX ON usage_logs (user_id);
CREATE INDEX ON usage_logs (app_id);
CREATE INDEX ON usage_logs (time DESC);
