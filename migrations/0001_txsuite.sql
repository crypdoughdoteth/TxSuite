CREATE TABLE IF NOT EXISTS Users (
    email VARCHAR(254) NOT NULL,
    password VARCHAR(120) NOT NULL,
    role ENUM('user', 'partner', 'admin') NOT NULL,
    PRIMARY KEY (email(16), role),
    UNIQUE INDEX (password(16))
);

CREATE TABLE IF NOT EXISTS Sui (
    userEmail VARCHAR(254) NOT NULL,
    object VARCHAR(66) NOT NULL,
    PRIMARY KEY (object(16)),
    KEY email_idx (userEmail(16))
);

CREATE TABLE IF NOT EXISTS Api (
    userEmail VARCHAR(254) NOT NULL,
    apiKey VARCHAR(66) NOT NULL,
    PRIMARY KEY (apiKey(16)),
    KEY email_idx (userEmail(16))
);
