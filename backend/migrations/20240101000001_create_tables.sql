-- Subject types enum
CREATE TYPE subject_type AS ENUM ('member', 'community_staff', 'platform_staff');

-- Subjects table
CREATE TABLE subjects (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(64) NOT NULL,
    display_name VARCHAR(128) NOT NULL,
    subject_type subject_type NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (username, subject_type)
);

CREATE INDEX idx_subjects_username_type ON subjects (username, subject_type);

-- Credential types enum
CREATE TYPE credential_type AS ENUM ('password', 'totp', 'passkey');

-- Credentials table (multi-credential per subject)
CREATE TABLE credentials (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    subject_id UUID NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    credential_type credential_type NOT NULL,
    -- For password: argon2 hash; For TOTP: encrypted secret; For passkey: JSON serialized data
    credential_data TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_credentials_subject ON credentials (subject_id);
CREATE UNIQUE INDEX idx_credentials_unique_type ON credentials (subject_id, credential_type)
    WHERE credential_type IN ('password', 'totp');

-- Sessions table (multi-device)
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    subject_id UUID NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    device_name VARCHAR(256),
    device_ip VARCHAR(45),
    user_agent TEXT,
    token_hash VARCHAR(128) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sessions_subject ON sessions (subject_id) WHERE is_active = TRUE;
CREATE INDEX idx_sessions_token ON sessions (token_hash) WHERE is_active = TRUE;
