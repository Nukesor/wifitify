ALTER TABLE stations
    ADD channel integer NOT NULL,
    ADD watch BOOLEAN DEFAULT FALSE NOT NULL
;


ALTER TABLE devices
    ADD watch BOOLEAN DEFAULT TRUE NOT NULL,
    DROP station
;

CREATE TABLE devices_stations (
    device integer,
    station integer,
    PRIMARY KEY (device, station),
    FOREIGN KEY (device) REFERENCES devices (id),
    FOREIGN KEY (station) REFERENCES stations (id)
);
