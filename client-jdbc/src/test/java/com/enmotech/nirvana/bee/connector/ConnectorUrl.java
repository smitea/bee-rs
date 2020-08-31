package com.enmotech.nirvana.bee.connector;

import java.util.Properties;

public class ConnectorUrl {
    final String ADDR = "127.0.0.1";
    //    final String ADDR = "116.63.140.185";
    final Integer PORT = 6142;

    public ClientInfo createClientRemoteInfo() {
        Properties properties = new Properties();
        properties.setProperty(ClientInfo.APPLICATION, "jdbc");
        properties.setProperty(ClientInfo.CONNECTION_TIMEOUT, "5");
        properties.setProperty(ClientInfo.CONNECTION_RESOURCE, "bee");
        properties.setProperty(ClientInfo.CONNECTION_PORT, "20002");
        properties.setProperty(ClientInfo.CONNECTION_HOST, "127.0.0.1");
        properties.setProperty(ClientInfo.CONNECTION_MODE, "password");
        properties.setProperty(ClientInfo.SESSION_MODE, "sqlite");
        properties.setProperty(ClientInfo.DATASOURCE_MODE, "remote");
        properties.setProperty(ClientInfo.USERNAME, "oracle");
        properties.setProperty(ClientInfo.PASSWORD, "admin");
        return new ClientInfo(ADDR, PORT, properties);
    }

    public ClientInfo createClientAgentInfo() {
        Properties properties = new Properties();
        properties.setProperty(ClientInfo.APPLICATION, "jdbc");
        properties.setProperty(ClientInfo.SESSION_MODE, "sqlite");
        properties.setProperty(ClientInfo.DATASOURCE_MODE, "agent");
        properties.setProperty(ClientInfo.CONNECTION_MODE, "default");
        return new ClientInfo(ADDR, PORT, properties);
    }

    public ClientInfo createClientAgentForLuaInfo() {
        Properties properties = new Properties();
        properties.setProperty(ClientInfo.APPLICATION, "jdbc");
        properties.setProperty(ClientInfo.SESSION_MODE, "lua");
        properties.setProperty(ClientInfo.DATASOURCE_MODE, "agent");
        properties.setProperty(ClientInfo.CONNECTION_MODE, "default");
        return new ClientInfo(ADDR, PORT, properties);
    }
}
