CREATE TABLE stations (
    id SERIAL,
    mac_address VARCHAR(17) NOT NULL,
    ssid VARCHAR(32),
    nickname Text DEFAULT NULL,
    description Text DEFAULT NULL,
    UNIQUE (nickname),
    UNIQUE (mac_address),
    PRIMARY KEY (id)
);
CREATE INDEX station_nickname ON stations (nickname);

CREATE TABLE devices (
    id SERIAL,
    mac_address VARCHAR(17) NOT NULL,
    nickname Text DEFAULT NULL,
    description Text DEFAULT NULL,
    station integer,
    FOREIGN KEY (station) REFERENCES stations (id),
    UNIQUE (nickname),
    UNIQUE (mac_address),
    PRIMARY KEY (id)
);
CREATE INDEX device_nickname ON devices (nickname);

CREATE TABLE data (
    time timestamp NOT NULL,
    device integer NOT NULL,
    station integer NOT NULL,
    bytes_per_minute integer,
    FOREIGN KEY (device) REFERENCES devices (id),
    FOREIGN KEY (station) REFERENCES stations (id),
    PRIMARY KEY (time, device, station)
);
CREATE INDEX time ON data (time, device);

SELECT create_hypertable('data', 'time');
