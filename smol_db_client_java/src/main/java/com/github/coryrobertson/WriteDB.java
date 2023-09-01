package com.github.coryrobertson;

import java.util.LinkedHashMap;
import java.util.Map;

public class WriteDB {

    public Map<String,String>[] Write = new Map[1];

    public WriteDB(String dbname, String location, String data) {
        Write[0] = new LinkedHashMap<>();
        Write[0].put("dbname",dbname);
        Write[0].put("location",location);
        Write[0].put("data",data);
    }

    // {"Write":[{"dbname":"test"},{"location":"test"},{"data":"data"}]}
}
