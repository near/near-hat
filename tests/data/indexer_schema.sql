CREATE TABLE
  indexers (
    id SERIAL PRIMARY KEY,
    event VARCHAR(255) NOT NULL,
    amount VARCHAR(255) NOT NULL,
    from_account VARCHAR(255),
    to_account VARCHAR(255),
    block_height INT
  );
