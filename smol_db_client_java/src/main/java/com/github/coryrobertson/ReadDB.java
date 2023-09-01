package com.github.coryrobertson;

import java.util.LinkedHashMap;
import java.util.Map;

public class ReadDB {

    public Map<String,String>[] Read = new Map[1];

    public ReadDB(String dbname, String location) {
        Read[0] = new LinkedHashMap<>();
        Read[0].put("dbname",dbname);
        Read[0].put("location",location);
    }

}
