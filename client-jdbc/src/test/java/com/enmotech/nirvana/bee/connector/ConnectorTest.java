package com.enmotech.nirvana.bee.connector;

import com.enmotech.nirvana.bee.connector.codec.BeeException;
import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertThrows;

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
        properties.setProperty(ClientInfo.CONNECTION_PORT, "49160");
        properties.setProperty(ClientInfo.CONNECTION_HOST, "127.0.0.1");
        properties.setProperty(ClientInfo.CONNECTION_MODE, "password");
        properties.setProperty(ClientInfo.SESSION_MODE, "sqlite");
        properties.setProperty(ClientInfo.DATASOURCE_MODE, "remote");
        properties.setProperty(ClientInfo.USERNAME, "oracle");
        properties.setProperty(ClientInfo.PASSWORD, "admin");
        return new ClientInfo(ADDR, PORT, properties);
    }

    private ClientInfo createClientInfoForConnectionRefused(){
        ClientInfo info = createClientInfo();
        info.getProperties().setProperty(ClientInfo.CONNECTION_PORT, "21");
        return info;
    }

    private ClientInfo createClientInfoForConnectionTimeout(){
        ClientInfo info = createClientInfo();
        info.getProperties().setProperty(ClientInfo.CONNECTION_HOST, "127.0.0.2");
        return info;
    }

    private ClientInfo createClientInfoForConnectionAuthUserFailed(){
        ClientInfo info = createClientInfo();
        info.getProperties().setProperty(ClientInfo.USERNAME, "oracle1");
        return info;
    }

    private ClientInfo createClientInfoForConnectionAuthPWDFailed(){
        ClientInfo info = createClientInfo();
        info.getProperties().setProperty(ClientInfo.PASSWORD, "admincs");
        return info;
    }

    @Test
    public void connectionRefused() throws SQLException {
        BeeException e = assertThrows(BeeException.class, () -> {
            new BeeConnection(createClientInfoForConnectionRefused());
        });
        assertEquals(192009, e.getCode());
    }

    @Test
    public void connectionTimeout() throws SQLException {
        BeeException e = assertThrows(BeeException.class, () -> {
            new BeeConnection(createClientInfoForConnectionTimeout());
        });
        assertEquals(192009, e.getCode());
    }

    @Test
    public void connectionAuthUserUserFailed() throws SQLException {
        BeeException e = assertThrows(BeeException.class, () -> {
            new BeeConnection(createClientInfoForConnectionAuthUserFailed());
        });
        assertEquals(126473, e.getCode());
    }

    @Test
    public void connectionAuthPWDFailed() throws SQLException {
        BeeException e = assertThrows(BeeException.class, () -> {
            new BeeConnection(createClientInfoForConnectionAuthPWDFailed());
        });
        assertEquals(126473, e.getCode());
    }
}