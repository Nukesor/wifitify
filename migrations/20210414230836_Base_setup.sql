CREATE TABLE stations (
    id SERIAL,
    mac_address VARCHAR(17),
    ssid VARCHAR(32),
    nickname Text,
    description Text,
    UNIQUE (nickname),
    UNIQUE (mac_address),
    PRIMARY KEY (id)
);
CREATE INDEX station_nickname ON stations (nickname);

CREATE TABLE devices (
    id SERIAL,
    mac_address VARCHAR(17),
    nickname Text,
    description Text,
    station integer,
    FOREIGN KEY (station) REFERENCES stations (id),
    UNIQUE (nickname),
    UNIQUE (mac_address),
    PRIMARY KEY (id)
);
CREATE INDEX device_nickname ON devices (nickname);

CREATE TABLE data (
    time timestamp,
    device integer,
    station integer,
    amount_per_minute integer,
    FOREIGN KEY (device) REFERENCES devices (id),
    FOREIGN KEY (station) REFERENCES stations (id),
    PRIMARY KEY (time, device, station)
);
CREATE INDEX time ON data (time, device);

SELECT create_hypertable('data', 'time');
