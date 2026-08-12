CREATE TABLE IF NOT EXISTS account_group (
    id BLOB NOT NULL PRIMARY KEY, -- UUID v7 stored as blob
    name TEXT NOT NULL UNIQUE CHECK(length(name) > 0),
    description TEXT NOT NULL DEFAULT ''
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS account (
    id BLOB NOT NULL PRIMARY KEY, -- UUID v7 stored as blob
    name TEXT NOT NULL CHECK(length(name) > 0),
    description TEXT NOT NULL DEFAULT '',
    kind INTEGER NOT NULL 
        CHECK(kind IN (0, 1, 2, 3, 4)), -- 0: Asset, 1: Liability, 2: Equity, 3: Income, 4: Expense
    currency INTEGER NOT NULL, -- 0: EUR, 1: USD, 2: RUB, 3: UAH, 4: BTC
    account_group_id BLOB, -- UUID v7 stored as blob; NULL when ungrouped
    FOREIGN KEY (account_group_id)
        REFERENCES account_group (id)
        ON DELETE SET NULL
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS journal_entry (
    id BLOB NOT NULL PRIMARY KEY, -- UUID v7 stored as blob
    date TEXT NOT NULL,           -- ISO-8601 date (YYYY-MM-DD)
    description TEXT NOT NULL DEFAULT ''
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS posting (
    journal_entry_id BLOB NOT NULL,
    account_id BLOB NOT NULL,
    amount INTEGER NOT NULL CHECK(amount <> 0), -- signed minor units: debit > 0, credit < 0;
        -- currency is account.currency
    FOREIGN KEY (journal_entry_id)
        REFERENCES journal_entry (id)
        ON DELETE CASCADE,
    FOREIGN KEY (account_id)
        REFERENCES account (id)
        ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_posting_journal_entry ON posting (journal_entry_id);
CREATE INDEX IF NOT EXISTS idx_posting_account ON posting (account_id);
