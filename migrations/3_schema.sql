CREATE TABLE registrant (
    ID SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    phone VARCHAR(13) NOT NULL,
    message TEXT,
    photo BYTEA -- BYTEA type is used to store binary data, including images
);
