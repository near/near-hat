CREATE TABLE
  indexers (
    id SERIAL PRIMARY KEY,
    functionName VARCHAR(255) NOT NULL,
    accountId VARCHAR(255) NOT NULL,
    methodName VARCHAR(255) NOT NULL,
    signerId VARCHAR(255) NOT NULL,
    block_height INT
  );
