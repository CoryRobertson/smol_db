package com.github.coryrobertson;

import java.io.IOException;

public class Main {
    public static void main(String[] args) throws IOException {

        final String key = "test_key_123";

        PacketBuilder packetBuilder = new PacketBuilder();

        packetBuilder.setKey(key);
        //packetBuilder.createDB("dbbbb", new String[]{"dasd"}, new String[]{"65465"}).replaceAll("\\\\", "")
        // {"CreateDB":[{"dbname":""dbbbb"","invalidation_time":"{"secs":30,"nanos":0}","can_others_rwx":"[false,false,false]","can_users_rwx":"[false,false,false]","admins":"["dasd"]","users":"["65465"]"}]}

        // {"CreateDB":[{"dbname":"test"},{"invalidation_time":{"secs":30,"nanos":0},"can_others_rwx":[false,true,false],"can_users_rwx":[true,true,true],"admins":["b"],"users":["a"]}]}
        // {"ListDBContents":{"dbname":"test"}}
        // {"Write":[{"dbname":"test"},{"location":"test"},{"data":"data"}]}
        // {"SetKey:"test_key_123"}


        SmolDBClient smdbClient = new SmolDBClient("localhost",8222);

        System.out.println("key set: " + smdbClient.setKey(key));


    }
}
