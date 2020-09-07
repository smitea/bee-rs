package com.enmotech.nirvana.bee.connector;

import com.enmotech.nirvana.bee.ConnectionFactory;
import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertThrows;

import java.sql.SQLException;

public class ConnectorTest extends ConnectionFactory {
    @Test
    public void connectionRefused() throws SQLException {
        BeeException e = assertThrows(BeeException.class, () -> {
            RemoteDatasource datasource = createRemoteDatasource(BeeDatasource.SessionMode.SQLITE);
            datasource.connectionProxy("127.0.0.1", 21);
            datasource.getConnection();
        });
        assertEquals(192009, e.getCode());
    }

    @Test
    public void connectionTimeout() throws SQLException {
        BeeException e = assertThrows(BeeException.class, () -> {
            RemoteDatasource datasource = createRemoteDatasource(BeeDatasource.SessionMode.SQLITE);
            datasource.connectionProxy("127.0.0.2", 22);
            datasource.getConnection();
        });
        assertEquals(192009, e.getCode());
    }

    @Test
    public void connectionAuthUserUserFailed() throws SQLException {
        BeeException e = assertThrows(BeeException.class, () -> {
            RemoteDatasource datasource = createRemoteDatasource(BeeDatasource.SessionMode.SQLITE);
            datasource.authPublicKey("sssss");
            datasource.getConnection();
        });
        assertEquals(60937, e.getCode());
    }

    @Test
    public void connectionAuthPWDFailed() throws SQLException {
        BeeException e = assertThrows(BeeException.class, () -> {
            RemoteDatasource datasource = createRemoteDatasource(BeeDatasource.SessionMode.SQLITE);
            datasource.authPassword(System.getProperty("user.name"), "ssss");
            datasource.getConnection();
        });
        assertEquals(126473, e.getCode());
    }
}