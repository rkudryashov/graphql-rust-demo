create table users (
    id serial primary key,
    username varchar not null unique,
    hash varchar(122) not null,
    first_name varchar(50) not null,
    last_name varchar(50) not null,
    role varchar(50) not null
);
