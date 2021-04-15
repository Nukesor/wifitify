CREATE TABLE stations (
    mac_address VARCHAR(17),
    ssid VARCHAR(32),
    nickname Text,
    description Text,
    UNIQUE (nickname),
    PRIMARY KEY (mac_address)
);
CREATE INDEX station_nickname ON stations (nickname);

CREATE TABLE devices (
    mac_address VARCHAR(17),
    nickname Text,
    description Text,
    station VARCHAR(17),
    FOREIGN KEY (station) REFERENCES stations (mac_address),
    UNIQUE (nickname),
    PRIMARY KEY (mac_address)
);
CREATE INDEX device_nickname ON devices (nickname);

CREATE TABLE data (
    time timestamp,
    device VARCHAR(17),
    station VARCHAR(17),
    frame_type VARCHAR,
    amount_per_minute Int,
    FOREIGN KEY (device) REFERENCES devices (mac_address),
    FOREIGN KEY (station) REFERENCES stations (mac_address),
    PRIMARY KEY (time, device, station)
);
CREATE INDEX time ON data (time, device);

SELECT create_hypertable('data', 'time');
