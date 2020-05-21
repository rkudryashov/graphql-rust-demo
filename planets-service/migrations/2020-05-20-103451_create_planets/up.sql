create table planets (
    id serial primary key,
    name varchar not null unique,
    planet_type varchar(20) not null
);

create table details (
    id serial primary key,
    mean_radius numeric(10,1) not null,
    mass numeric(30) not null,
    population numeric(10,1),
    planet_id integer references planets not null
);
