package com.enmotech.nirvana.bee.connector;

public class AgentDatasource extends BeeDatasource {
    static String CONNECT_MODE = "default";

    public AgentDatasource(String host, int port) {
        super(host, port);

        setConnectionMode(CONNECT_MODE);
        setDataSourceMode("agent");
        setProxyHost("127.0.0.1");
        setProxyPort(6142);
        setUsername(System.getProperty("user.name"));
        setResource("bee");
    }
}
