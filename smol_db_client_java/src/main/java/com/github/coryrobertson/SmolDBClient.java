package com.github.coryrobertson;

import java.io.*;
import java.net.Socket;

public class SmolDBClient {

    private final String SUCCESS = "SuccessNoData";

    public Socket clientSocket;
    public BufferedReader br;
    public BufferedWriter bw;

    private final PacketBuilder pb = new PacketBuilder();

    public SmolDBClient(String ip, int port) throws IOException {
        clientSocket = new Socket(ip,port);
        br = new BufferedReader(new InputStreamReader(clientSocket.getInputStream()));
        bw = new BufferedWriter(new OutputStreamWriter(clientSocket.getOutputStream()));
    }

    public boolean setKey(String key) throws IOException {
        bw.write(pb.setKey(key));
        bw.flush();
        return this.read().contains(SUCCESS);
    }

    public boolean writeDB(String location, String data) {
//        String packet = this.packetBuilder()
        return false;
    }


    public String packetBuilder(String packetName, String packetContent) {
        return "{\"" + packetName + "\":\"" + packetContent + "\"}";
    }

    public String read() throws IOException {
        var carr = new char[1024];
        int readlen = this.br.read(carr);
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < readlen; i++) {
            sb.append(carr[i]);
        }
        return sb.toString();
    }

}
