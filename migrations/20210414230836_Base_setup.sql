CREATE TABLE stations (
    id SERIAL,
    mac_address VARCHAR(17) NOT NULL,
    ssid VARCHAR(32),
    channel integer NOT NULL,
    power_level integer,
    watch BOOLEAN DEFAULT FALSE NOT NULL,
    nickname Text DEFAULT NULL,
    description Text DEFAULT NULL,
    PRIMARY KEY (id),
    UNIQUE (nickname),
    UNIQUE (mac_address)
);
CREATE INDEX station_nickname ON stations (nickname);

CREATE TABLE devices (
    id SERIAL,
    mac_address VARCHAR(17) NOT NULL,
    watch BOOLEAN DEFAULT TRUE NOT NULL,
    nickname Text DEFAULT NULL,
    description Text DEFAULT NULL,
    PRIMARY KEY (id),
    UNIQUE (nickname),
    UNIQUE (mac_address)
);
CREATE INDEX device_nickname ON devices (nickname);

CREATE TABLE devices_stations (
    device integer,
    station integer,
    PRIMARY KEY (device, station),
    FOREIGN KEY (device) REFERENCES devices (id) ON DELETE CASCADE,
    FOREIGN KEY (station) REFERENCES stations (id) ON DELETE CASCADE
);

CREATE TABLE data (
    time timestamp with time zone NOT NULL,
    device integer NOT NULL,
    station integer NOT NULL,
    bytes_per_minute integer,
    PRIMARY KEY (time, device, station),
    FOREIGN KEY (device) REFERENCES devices (id) ON DELETE CASCADE,
    FOREIGN KEY (station) REFERENCES stations (id) ON DELETE CASCADE
);
CREATE INDEX time ON data (time, device);

SELECT create_hypertable('data', 'time');
