package com.github.coryrobertson;

import com.google.gson.Gson;

public class PacketBuilder {

    Gson gson = new Gson();

    public PacketBuilder() {}

    public String writeDB(String dbname, String location, String data) {
        WriteDB writeDB = new WriteDB(dbname,location,data);
        return gson.toJson(writeDB);
    }

    public String createDB(String dbname, String[] admins, String[] users) {
        CreateDB createDB = new CreateDB(dbname,admins,users);
        return gson.toJson(createDB).replaceAll("\\\\", "");
    }


    public String readDB(String dbname, String location) {
        ReadDB readDB = new ReadDB(dbname,location);
        return gson.toJson(readDB);
    }

    public String setKey(String key) {
        SetKey setkey = new SetKey(key);
        return gson.toJson(setkey);
    }
}
