package com.enmotech.nirvana.bee.connector;

import com.enmotech.nirvana.bee.connector.codec.BeeException;
import org.junit.Test;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.Properties;

public class ConnectorTest {
    final String ADDR = "127.0.0.1";
    final Integer PORT = 6142;

    private ClientInfo createClientInfo() {
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

    @Test
    public void connection() throws SQLException {
        Connection connection = new BeeConnection(createClientInfo());
        assert connection.isClosed();
    }
}