package com.github.coryrobertson;

import com.google.gson.Gson;

import java.util.LinkedHashMap;
import java.util.Map;

public class CreateDB {
    // {"CreateDB":[{"dbname":"test"},{"invalidation_time":{"secs":30,"nanos":0},"can_others_rwx":[false,true,false],"can_users_rwx":[true,true,true],"admins":["b"],"users":["a"]}]}
    public Map<String,String>[] CreateDB = new Map[1];

    public CreateDB(String dbname,String[] admins,String[] users) {

        Gson gson = new Gson();
        this.CreateDB[0] = new LinkedHashMap<>();
//        var dbname_map = new LinkedHashMap<String,String>();
//        dbname_map.put("dbname",dbname);
        CreateDB[0].put("dbname",gson.toJson(dbname));
        var invaltime = new LinkedHashMap<String,Integer>();
        invaltime.put("secs",30);
        invaltime.put("nanos",0);
        CreateDB[0].put("invalidation_time",gson.toJson(invaltime));
        boolean[] can_others_rwx = {false, false, false};
        boolean[] can_users_rwx = {false, false, false};
        CreateDB[0].put("can_others_rwx", gson.toJson(can_others_rwx));
        CreateDB[0].put("can_users_rwx", gson.toJson(can_users_rwx));
        CreateDB[0].put("admins", gson.toJson(admins));
        CreateDB[0].put("users", gson.toJson(users));
    }

}
