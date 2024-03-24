-- CREATE SCHEMA named 'test_schema' --
CREATE SCHEMA test_schema;
SET search_path TO test_schema;
SET TIME ZONE 'Asia/Tokyo';

-- CREATE TABLES --
CREATE TABLE users
(
    id                serial PRIMARY KEY,
    username          varchar   not null unique,
    firstname         varchar   not null,
    lastname          varchar   not null,
    phone_number      varchar   not null unique,
    created_timestamp timestamp,
    updated_timestamp timestamp,
    updated_numbers   integer            DEFAULT 0,
    last_login        timestamp,
    is_deleted        bool      not null DEFAULT FALSE
);

CREATE TABLE categories
(
    id         serial PRIMARY KEY,
    user_id    integer not null,
    category   varchar not null unique,
    is_deleted bool    not null DEFAULT FALSE,
    FOREIGN KEY (user_id) REFERENCES users (id)
);

CREATE TABLE subcategories
(
    id          serial PRIMARY KEY,
    category_id integer not null,
    subcategory varchar not null,
    is_deleted  bool    not null DEFAULT FALSE,
    FOREIGN KEY (category_id) REFERENCES categories (id)
);

CREATE TABLE records
(
    id             serial PRIMARY KEY,
    user_id        integer not null,
    record_date    date    not null,
    subcategory_id integer not null,
    work_time      real  not null,
    message_comment        varchar,
    FOREIGN KEY (user_id) REFERENCES users (id),
    FOREIGN KEY (subcategory_id) REFERENCES subcategories (id)
);

-- CREATE USER and GRANT AUTHORITY --
CREATE ROLE app WITH LOGIN PASSWORD 'password';

SET search_path TO test_schema;
SET TIME ZONE 'Asia/Tokyo';

GRANT USAGE ON SCHEMA test_schema TO app;
GRANT USAGE, SELECT ON SEQUENCE users_id_seq TO app;
GRANT USAGE, SELECT ON SEQUENCE categories_id_seq TO app;
GRANT USAGE, SELECT ON SEQUENCE subcategories_id_seq TO app;
GRANT USAGE, SELECT ON SEQUENCE records_id_seq TO app;

GRANT SELECT, INSERT, UPDATE, DELETE ON records TO app;
GRANT SELECT ON users TO app;
GRANT SELECT ON categories TO app;
GRANT SELECT ON subcategories TO app;

-- CREATE FUNCTIONS --
CREATE OR REPLACE FUNCTION update_auto_process()
RETURNS TRIGGER AS $$
BEGIN
    SET TIME ZONE 'UTC';
    NEW.updated_numbers = COALESCE(OLD.updated_numbers, 0) + 1;
    NEW.updated_timestamp = NOW();
    RETURN NEW;
END
$$ LANGUAGE plpgsql;
CREATE TRIGGER update_process_trigger
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION update_auto_process();

CREATE OR REPLACE FUNCTION insert_auto_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    SET TIME ZONE 'UTC';
    NEW.created_timestamp = NOW();
    RETURN NEW;
END
$$ LANGUAGE plpgsql;
CREATE TRIGGER auto_created_timestamp_trigger
BEFORE INSERT ON users
FOR EACH ROW
EXECUTE FUNCTION insert_auto_timestamp();


-- CREATE DATA --
INSERT INTO
    users (username, firstname, lastname, phone_number, updated_timestamp, last_login)
VALUES
    ('user1', 'taro', 'postgres', '000-0000-0000', NULL, NULL),
    ('user2', 'jiro', 'validate', '111-1111-1111', NULL, NULL),
    ('user3', 'hanako', 'application', '222-2222-2222', NULL, NULL),
    ('user4', 'saburo', 'test', '333-3333-3333', NULL, NULL);

INSERT INTO
    categories (user_id, category)
VALUES
    (1, 'category1'),
    (2, 'category2'),
    (1, 'category3'),
    (3, 'category4'),
    (4, 'category5'),
    (2, 'category6');

INSERT INTO
    subcategories (category_id, subcategory)
VALUES
    (1, 'subcategory1'),
    (2, 'subcategory2'),
    (4, 'subcategory3'),
    (3, 'subcategory4'),
    (2, 'subcategory5'),
    (1, 'subcategory6'),
    (4, 'subcategory7'),
    (3, 'subcategory8'),
    (2, 'subcategory9'),
    (1, 'subcategory10');

INSERT INTO
    records (user_id, record_date, subcategory_id, work_time, message_comment)
VALUES
    (1, '2024-02-24', 1, 4.5, ''),
    (2, '2024-02-25', 3, 3., ''),
    (4, '2024-02-22', 4, 2.8, ''),
    (1, '2024-02-23', 2, 5.3, ''),
    (2, '2024-02-25', 4, 2.3, ''),
    (3, '2024-02-24', 8, 2.9, ''),
    (4, '2024-02-23', 7, 1.3, ''),
    (2, '2024-01-24', 10, 14.5, 'Over the default time'),
    (1, '2024-02-14', 9, 8.2, ''),
    (3, '2024-02-23', 3, 4.6, ''),
    (3, '2023-12-14', 2, 10.3, ''),
    (2, '2024-03-24', 1, 3.8, ''),
    (4, '2024-01-04', 8, 11.5, ''),
    (2, '2024-02-20', 6, 3.5, ''),
    (1, '2024-03-01', 5, 2.1, ''),
    (1, '2023-07-24', 4, 7.3, '');
