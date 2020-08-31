package com.enmotech.nirvana.bee.connector;

import com.enmotech.nirvana.bee.connector.codec.BeeException;
import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertThrows;

import java.sql.SQLException;
import java.util.Properties;

public class ConnectorTest extends ConnectorUrl {

    private ClientInfo createClientInfoForConnectionRefused() {
        ClientInfo info = createClientRemoteInfo();
        info.getProperties().setProperty(ClientInfo.CONNECTION_PORT, "21");
        return info;
    }

    private ClientInfo createClientInfoForConnectionTimeout() {
        ClientInfo info = createClientRemoteInfo();
        info.getProperties().setProperty(ClientInfo.CONNECTION_HOST, "127.0.0.2");
        return info;
    }

    private ClientInfo createClientInfoForConnectionAuthUserFailed() {
        ClientInfo info = createClientRemoteInfo();
        info.getProperties().setProperty(ClientInfo.USERNAME, "oracle1");
        return info;
    }

    private ClientInfo createClientInfoForConnectionAuthPWDFailed() {
        ClientInfo info = createClientRemoteInfo();
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